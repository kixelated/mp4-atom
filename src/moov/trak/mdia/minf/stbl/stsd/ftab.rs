use crate::*;

/// Font table for tx3g
///
/// 3GPP TS 26.245 or ETSI TS 126 245 Section 5.16
/// See https://www.etsi.org/deliver/etsi_ts/126200_126299/126245/18.00.00_60/ts_126245v180000p.pdf

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FontEntry {
    pub font_id: u16,
    pub font: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ftab {
    pub font_entries: Vec<FontEntry>,
}

impl Atom for Ftab {
    const KIND: FourCC = FourCC::new(b"ftab");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let entry_count = u16::decode(buf)?;
        let mut font_entries = Vec::with_capacity(std::cmp::min(8, entry_count as usize));
        for _ in 0..entry_count {
            let font_id = u16::decode(buf)?;
            let font = decode_length_prefixed_bom_string::<u8>(buf)?;
            let font_entry = FontEntry { font_id, font };
            font_entries.push(font_entry);
        }
        Ok(Ftab { font_entries })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let entry_count: u16 = self
            .font_entries
            .len()
            .try_into()
            .map_err(|_| Error::TooLarge(Self::KIND))?;
        entry_count.encode(buf)?;
        for font_entry in &self.font_entries {
            font_entry.font_id.encode(buf)?;
            // We always encode as UTF-8, so there won't be round tripping of UTF-16. That is OK.
            encode_length_prefixed_string::<u8>(buf, &font_entry.font, Self::KIND)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ftab_utf16_font_names_decode_as_utf8_on_encode() {
        let utf16le = [b"\xFF\xFE".as_slice(), b"S\0e\0r\0i\0f\0"].concat();
        let utf16be = [b"\xFE\xFF".as_slice(), b"\0S\0a\0n\0s"].concat();

        let mut body = vec![0, 2];
        for (id, name) in [(1u8, &utf16le), (2, &utf16be)] {
            body.extend([0, id, name.len() as u8]);
            body.extend(name);
        }

        let ftab = Ftab::decode_body(&mut body.as_slice()).unwrap();
        assert_eq!(
            ftab.font_entries,
            vec![
                FontEntry {
                    font_id: 1,
                    font: "Serif".into()
                },
                FontEntry {
                    font_id: 2,
                    font: "Sans".into()
                },
            ]
        );

        // UTF-16 isn't preserved: names are re-encoded as UTF-8.
        let mut encoded = Vec::new();
        ftab.encode_body(&mut encoded).unwrap();
        assert_eq!(encoded, b"\0\x02\0\x01\x05Serif\0\x02\x04Sans");
    }

    #[test]
    fn test_ftab_font_name_exceeds_buffer() {
        let err = Ftab::decode_body(&mut [0, 1, 0, 1, 9, b'a'].as_slice()).unwrap_err();
        assert!(matches!(err, Error::OutOfBounds));
    }
}
