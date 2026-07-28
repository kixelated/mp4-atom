mod cprt;
mod kind;
mod rtng;
mod skip;

pub use cprt::*;
pub use kind::*;
pub use rtng::*;
pub use skip::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Udta {
    /// Zero or more, one per language (ISO/IEC 14496-12 §8.10.2).
    pub cprt: Vec<Cprt>,
    /// Zero or more, one per role/kind label (ISO/IEC 14496-12 §8.10.4).
    pub kind: Vec<Kind>,
    /// Zero or one (ISO/IEC 14496-12 §8.11.1).
    pub meta: Option<Meta>,
    /// Zero or more, one per language (3GPP TS 26.244 clause 8).
    pub rtng: Vec<Rtng>,
}

impl Atom for Udta {
    const KIND: FourCC = FourCC::new(b"udta");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let mut cprt = Vec::new();
        let mut kind = Vec::new();
        let mut meta = None;
        let mut rtng = Vec::new();

        // `udta` is a free-form user-data container. QuickTime writes a
        // track-name `name` box here, whose fourcc collides with the iTunes
        // `ilst` Name item in the global atom table, so dispatch children by
        // header against the udta namespace only — the same shape as
        // `Ilst::decode_body` — rather than through `Any`. Unknown children go
        // to `decode_unknown` (a warning by default, an error under strict).
        while let Some(header) = Header::decode_maybe(buf)? {
            let size = header.size.unwrap_or(buf.remaining());
            if size > buf.remaining() {
                // A child whose declared size exceeds what remains is truncated.
                return Err(Error::OutOfBounds);
            }
            // Multiplicity follows the specs: `cprt` and `kind` are "zero or
            // more" (ISO/IEC 14496-12 §8.10.2 — one copyright notice per
            // language — and §8.10.4 — one box per role value), and 3GPP
            // TS 26.244 clause 8 admits each asset box (`rtng` included) once
            // per language, so repeats accumulate. `meta` is "zero or one" per
            // container (ISO/IEC 14496-12 §8.11.1), so a second one is
            // malformed.
            match header.kind {
                Cprt::KIND => cprt.push(Cprt::decode_atom(&header, buf)?),
                Kind::KIND => kind.push(Kind::decode_atom(&header, buf)?),
                Meta::KIND => {
                    if meta.is_some() {
                        return Err(Error::DuplicateBox(Meta::KIND));
                    }
                    meta = Some(Meta::decode_atom(&header, buf)?);
                }
                Rtng::KIND => rtng.push(Rtng::decode_atom(&header, buf)?),
                // `free`/`skip` are padding boxes; drop them.
                Free::KIND | Skip::KIND => {
                    buf.advance(size);
                }
                unknown => {
                    let body = Vec::decode(&mut buf.slice(size))?;
                    buf.advance(size);
                    Self::decode_unknown(&Any::Unknown(unknown, body))?;
                }
            }
        }
        // Drop any sub-header padding remainder after the children.
        skip_trailing_padding(buf);

        Ok(Udta {
            cprt,
            kind,
            meta,
            rtng,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.cprt.encode(buf)?;
        self.meta.encode(buf)?;
        self.kind.encode(buf)?;
        self.rtng.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_udta_empty() {
        let expected = Udta {
            cprt: vec![],
            meta: None,
            kind: vec![],
            rtng: vec![],
        };

        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let output = Udta::decode(&mut buf).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_udta() {
        let expected = Udta {
            cprt: vec![Cprt {
                language: "und".into(),
                notice: "MIT or Apache".into(),
            }],
            meta: Some(Meta {
                hdlr: Hdlr {
                    handler: FourCC::new(b"fake"),
                    name: "".into(),
                },
                items: vec![],
            }),
            kind: vec![Kind {
                scheme_uri: "http://www.w3.org/TR/html5/".into(),
                value: "".into(),
            }],
            rtng: vec![Rtng {
                entity: b"BBFC".into(),
                criteria: b"PG13".into(),
                language: "eng".into(),
                rating_info: "test info".into(),
            }],
        };

        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let output = Udta::decode(&mut buf).unwrap();
        assert_eq!(output, expected);
    }

    // From MPEG File Format Conformance, isobmff/02_dref_edts_img.mp4
    const ENCODED_UDTA_WITH_CPRT: &[u8] = &[
        0x00, 0x00, 0x00, 0x70, 0x75, 0x64, 0x74, 0x61, 0x00, 0x00, 0x00, 0x68, 0x63, 0x70, 0x72,
        0x74, 0x00, 0x00, 0x00, 0x00, 0x55, 0xc4, 0x45, 0x4e, 0x53, 0x54, 0x20, 0x49, 0x73, 0x6f,
        0x4d, 0x65, 0x64, 0x69, 0x61, 0x20, 0x43, 0x6f, 0x6e, 0x66, 0x6f, 0x72, 0x6d, 0x61, 0x6e,
        0x63, 0x65, 0x20, 0x46, 0x69, 0x6c, 0x65, 0x73, 0x20, 0x2d, 0x20, 0x45, 0x4e, 0x53, 0x54,
        0x20, 0x28, 0x63, 0x29, 0x20, 0x32, 0x30, 0x30, 0x36, 0x20, 0x2d, 0x20, 0x52, 0x69, 0x67,
        0x68, 0x74, 0x73, 0x20, 0x72, 0x65, 0x6c, 0x65, 0x61, 0x73, 0x65, 0x64, 0x20, 0x66, 0x6f,
        0x72, 0x20, 0x49, 0x53, 0x4f, 0x20, 0x43, 0x6f, 0x6e, 0x66, 0x6f, 0x72, 0x6d, 0x61, 0x6e,
        0x63, 0x65, 0x20, 0x75, 0x73, 0x65, 0x00,
    ];

    #[test]
    fn test_udta_cprt() {
        let mut buf = std::io::Cursor::new(ENCODED_UDTA_WITH_CPRT);

        let udta = Udta::decode(&mut buf).expect("failed to decode udta");

        assert_eq!(
            udta,
            Udta {
                cprt: vec![Cprt { language: "und".into(), notice: "ENST IsoMedia Conformance Files - ENST (c) 2006 - Rights released for ISO Conformance use".into() }],
                ..Default::default()
            }
        );

        let mut buf = Vec::new();
        udta.encode(&mut buf).unwrap();

        assert_eq!(buf, ENCODED_UDTA_WITH_CPRT);
    }

    // From MPEG File Format Conformance, nalu/hevc/hev1_clg1_header.mp4
    const ENCODED_UDTA_WITH_KIND: &[u8] = &[
        0x00, 0x00, 0x00, 0x31, 0x75, 0x64, 0x74, 0x61, 0x00, 0x00, 0x00, 0x29, 0x6b, 0x69, 0x6e,
        0x64, 0x00, 0x00, 0x00, 0x00, 0x75, 0x72, 0x6e, 0x3a, 0x6d, 0x70, 0x65, 0x67, 0x3a, 0x64,
        0x61, 0x73, 0x68, 0x3a, 0x72, 0x6f, 0x6c, 0x65, 0x3a, 0x32, 0x30, 0x31, 0x31, 0x00, 0x6d,
        0x61, 0x69, 0x6e, 0x00,
    ];

    #[test]
    fn test_udta_kind() {
        let mut buf = std::io::Cursor::new(ENCODED_UDTA_WITH_KIND);

        let udta = Udta::decode(&mut buf).expect("failed to decode udta");

        assert_eq!(
            udta,
            Udta {
                kind: vec![Kind {
                    scheme_uri: "urn:mpeg:dash:role:2011".into(),
                    value: "main".into()
                }],
                ..Default::default()
            }
        );

        let mut buf = Vec::new();
        udta.encode(&mut buf).unwrap();

        assert_eq!(buf, ENCODED_UDTA_WITH_KIND);
    }

    // A track-level QuickTime `udta` holding an empty (4-null-byte) track-name
    // `name` box. Its fourcc collides with the iTunes `ilst` Name item in the
    // global atom table.
    const ENCODED_UDTA_WITH_QT_NAME: &[u8] = &[
        0x00, 0x00, 0x00, 0x14, 0x75, 0x64, 0x74, 0x61, // udta, size 20
        0x00, 0x00, 0x00, 0x0c, 0x6e, 0x61, 0x6d, 0x65, // name, size 12
        0x00, 0x00, 0x00, 0x00, // body: empty QuickTime track-name string
    ];

    // A sub-header padding remainder after a `udta`'s children must be skipped
    // as padding while the real children still decode.
    #[test]
    fn test_udta_trailing_padding() {
        let mut buf = ENCODED_UDTA_WITH_CPRT.to_vec();
        buf.extend_from_slice(&[0, 0, 0, 0]);
        let size = (buf.len() as u32).to_be_bytes();
        buf[0..4].copy_from_slice(&size);

        let udta = Udta::decode(&mut buf.as_slice()).expect("trailing padding must be tolerated");
        assert_eq!(
            udta.cprt.len(),
            1,
            "the real child still decodes, the padding is dropped"
        );
    }

    #[test]
    fn test_udta_quicktime_name_not_misparsed() {
        // The udta-namespace dispatch must NOT route `name` to the global
        // `Any` table's ilst Name item (which would UnderDecode the 4-byte
        // body). Under `cfg(test)` an unknown child is an error, proving the
        // atom is treated as unknown-to-udta rather than misparsed as ilst.
        let mut buf = std::io::Cursor::new(ENCODED_UDTA_WITH_QT_NAME);
        match Udta::decode(&mut buf) {
            Err(Error::UnexpectedBox(kind)) => assert_eq!(kind, FourCC::new(b"name")),
            other => panic!("expected UnexpectedBox(name), got {other:?}"),
        }
    }

    // Repeated `cprt`/`kind`/`rtng` children are spec-legal — ISO/IEC 14496-12
    // declares both CopyrightBox (§8.10.2) and KindBox (§8.10.4) as quantity
    // "zero or more" (one copyright per language, one kind per role value),
    // and 3GPP TS 26.244 clause 8 admits "zero or more sub-boxes of each kind,
    // zero or one for each language" of the asset boxes, `rtng` included.
    // Repeats must accumulate — not error, not overwrite each other.
    #[test]
    fn test_udta_repeated_children() {
        let expected = Udta {
            cprt: vec![
                Cprt {
                    language: "eng".into(),
                    notice: "All rights reserved".into(),
                },
                Cprt {
                    language: "fra".into(),
                    notice: "Tous droits réservés".into(),
                },
            ],
            kind: vec![
                Kind {
                    scheme_uri: "urn:mpeg:dash:role:2011".into(),
                    value: "main".into(),
                },
                Kind {
                    scheme_uri: "urn:mpeg:dash:role:2011".into(),
                    value: "caption".into(),
                },
            ],
            meta: None,
            rtng: vec![
                Rtng {
                    entity: b"BBFC".into(),
                    criteria: b"PG13".into(),
                    language: "eng".into(),
                    rating_info: "parental guidance".into(),
                },
                Rtng {
                    entity: b"BBFC".into(),
                    criteria: b"PG13".into(),
                    language: "fra".into(),
                    rating_info: "accord parental".into(),
                },
            ],
        };

        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let output = Udta::decode(&mut buf).unwrap();
        assert_eq!(output, expected);
    }

    // `meta` is quantity "zero or one" per container (ISO/IEC 14496-12
    // §8.11.1) — unlike the repeatable children above, a second one is
    // malformed and must be rejected, not silently overwritten.
    #[test]
    fn test_udta_duplicate_meta() {
        let meta = Meta {
            hdlr: Hdlr {
                handler: FourCC::new(b"fake"),
                name: "".into(),
            },
            items: vec![],
        };
        let mut body = Vec::new();
        meta.encode(&mut body).unwrap();
        meta.encode(&mut body).unwrap();

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&((body.len() + 8) as u32).to_be_bytes());
        bytes.extend_from_slice(b"udta");
        bytes.extend_from_slice(&body);

        match Udta::decode(&mut bytes.as_slice()) {
            Err(Error::DuplicateBox(kind)) => assert_eq!(kind, Meta::KIND),
            other => panic!("expected DuplicateBox(meta), got {other:?}"),
        }
    }
}
