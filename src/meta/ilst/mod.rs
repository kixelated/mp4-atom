mod covr;
mod cprt;
mod data;
mod desc;
mod name;
mod tool;
mod year;

pub use covr::*;
pub use cprt::*;
pub use data::*;
pub use desc::*;
pub use name::*;
pub use tool::*;
pub use year::*;

use crate::*;

/// Below this value, a child's raw fourcc looks like a big-endian 1-based
/// index (top byte zero) rather than 4 ASCII/Latin1 tag characters — the
/// scheme Apple's `mdta`-keyed metadata (see [`Keys`]) uses for `ilst` items.
const MDTA_INDEX_LIMIT: u32 = 0x0100_0000;

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ilst {
    pub name: Option<Name>,
    pub year: Option<Year>, // Called day in the spec
    pub covr: Option<Covr>,
    pub desc: Option<Desc>,
    pub ctoo: Option<Tool>,      // 4CC: "©too"
    pub cprt: Option<Copyright>, // iTunes item, NOT the ISO CopyrightBox

    /// Apple `mdta`-keyed items, keyed by their 1-based index into the
    /// sibling [`Keys`] box's entries.
    pub mdta: Vec<(u32, IlstData)>,
}

impl Atom for Ilst {
    const KIND: FourCC = FourCC::new(b"ilst");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let mut name = None;
        let mut year = None;
        let mut covr = None;
        let mut desc = None;
        let mut ctoo = None;
        let mut cprt = None;
        let mut mdta = vec![];

        // `ilst` children live in the iTunes metadata namespace, which reuses
        // fourccs of unrelated ISO atoms — an ilst `cprt` item wraps a `data`
        // atom while the ISO CopyrightBox is a FullBox with a language code.
        // Dispatch by header against the ilst namespace only, never through
        // the global `Any` table.
        while let Some(header) = Header::decode_maybe(buf)? {
            let size = header.size.unwrap_or(buf.remaining());
            if size > buf.remaining() {
                // Truncated child: stop and let the caller's remaining-bytes
                // check report it, matching `Any::decode_maybe`.
                break;
            }
            match header.kind {
                Name::KIND => name = Some(Name::decode_atom(&header, buf)?),
                Year::KIND => year = Some(Year::decode_atom(&header, buf)?),
                Covr::KIND => covr = Some(Covr::decode_atom(&header, buf)?),
                Desc::KIND => desc = Some(Desc::decode_atom(&header, buf)?),
                Tool::KIND => ctoo = Some(Tool::decode_atom(&header, buf)?),
                Copyright::KIND => cprt = Some(Copyright::decode_atom(&header, buf)?),
                kind => {
                    let index = u32::from(kind);

                    let mdta_item = if index < MDTA_INDEX_LIMIT {
                        IlstData::decode(&mut buf.slice(size)).ok()
                    } else {
                        None
                    };

                    if let Some(data) = mdta_item {
                        mdta.push((index, data))
                    } else {
                        let body = Vec::decode(&mut buf.slice(size))?;
                        Self::decode_unknown(&Any::Unknown(kind, body))?;
                    }

                    buf.advance(size);
                }
            }
        }

        Ok(Ilst {
            name,
            year,
            covr,
            desc,
            ctoo,
            cprt,
            mdta,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.name.encode(buf)?;
        self.year.encode(buf)?;
        self.covr.encode(buf)?;
        self.desc.encode(buf)?;
        self.ctoo.encode(buf)?;
        self.cprt.encode(buf)?;

        for (index, data) in &self.mdta {
            let start = buf.len();
            0u32.encode(buf)?; // size placeholder
            FourCC::from(*index).encode(buf)?;
            data.encode(buf)?;

            let size: u32 = (buf.len() - start)
                .try_into()
                .map_err(|_| Error::TooLarge(FourCC::from(*index)))?;
            buf.set_slice(start, &size.to_be_bytes());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ilst() {
        let expected = Ilst {
            year: Year("src_year".to_string()).into(),
            ..Default::default()
        };

        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Ilst::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_ilst_empty() {
        let expected = Ilst::default();
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Ilst::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    // Build `[size:u32][fourcc][body]`.
    fn atom_box(fourcc: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut v = Vec::with_capacity(8 + body.len());
        v.extend_from_slice(&((8 + body.len()) as u32).to_be_bytes());
        v.extend_from_slice(fourcc);
        v.extend_from_slice(body);
        v
    }

    // An iTunes `cprt` item (as written by FFmpeg for `-metadata copyright`)
    // wraps a `data` atom. Pre-fix, Ilst dispatched it through the global
    // `Any` table where the fourcc collides with the ISO CopyrightBox, so the
    // whole decode failed with UnderDecode(cprt).
    #[test]
    fn test_ilst_itunes_cprt() {
        let text = b"(c) 2026 Example";
        let mut data_body = Vec::new();
        data_body.extend_from_slice(&1u32.to_be_bytes()); // type indicator: UTF-8
        data_body.extend_from_slice(&0u32.to_be_bytes()); // country + language
        data_body.extend_from_slice(text);
        let cprt_item = atom_box(b"cprt", &atom_box(b"data", &data_body));
        let encoded = atom_box(b"ilst", &cprt_item);

        let decoded = Ilst::decode(&mut encoded.as_slice()).unwrap();
        assert_eq!(
            decoded,
            Ilst {
                cprt: Some(Copyright {
                    country_indicator: 0,
                    language_indicator: 0,
                    text: "(c) 2026 Example".into(),
                }),
                ..Default::default()
            }
        );

        // The encoder writes the same long-style `data` layout back.
        let mut reencoded = Vec::new();
        decoded.encode(&mut reencoded).unwrap();
        assert_eq!(reencoded, encoded);
    }

    // Unknown ilst items still go through decode_unknown (an error in
    // strict/test builds, a warning otherwise).
    #[test]
    fn test_ilst_unknown_item() {
        let mut data_body = Vec::new();
        data_body.extend_from_slice(&1u32.to_be_bytes());
        data_body.extend_from_slice(&0u32.to_be_bytes());
        data_body.extend_from_slice(b"Some Artist");
        let art_item = atom_box(b"\xa9ART", &atom_box(b"data", &data_body));
        let encoded = atom_box(b"ilst", &art_item);

        let result = Ilst::decode(&mut encoded.as_slice());
        match result {
            Err(Error::UnexpectedBox(kind)) => assert_eq!(kind, FourCC::new(b"\xa9ART")),
            other => panic!("expected UnexpectedBox, got {other:?}"),
        }
    }

    // Apple's `mdta`-keyed metadata scheme (used alongside a sibling `keys`
    // box) keys each `ilst` item by a raw 1-based index rather than a
    // well-known fourcc — the index is stored where a fourcc would be, with
    // its top byte zero (so it can't collide with a real ASCII/Latin1 tag).
    #[test]
    fn test_ilst_mdta_item() {
        let mut data_body = Vec::new();
        data_body.extend_from_slice(&1u32.to_be_bytes()); // type indicator: UTF-8
        data_body.extend_from_slice(&0u32.to_be_bytes()); // country + language
        data_body.extend_from_slice(b"Apple");
        let item = atom_box(&1u32.to_be_bytes(), &atom_box(b"data", &data_body));
        let encoded = atom_box(b"ilst", &item);

        let decoded = Ilst::decode(&mut encoded.as_slice()).unwrap();
        assert_eq!(
            decoded,
            Ilst {
                mdta: vec![(
                    1,
                    IlstData {
                        country_indicator: 0,
                        language_indicator: 0,
                        value: IlstDataValue::Utf8("Apple".into()),
                    }
                )],
                ..Default::default()
            }
        );

        // The encoder writes the same long-style `data` layout back.
        let mut reencoded = Vec::new();
        decoded.encode(&mut reencoded).unwrap();
        assert_eq!(reencoded, encoded);
    }
}
