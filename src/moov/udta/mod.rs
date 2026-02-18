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
    pub cprt: Option<Cprt>,
    pub kind: Option<Kind>,
    pub meta: Option<Meta>,
    pub rtng: Option<Rtng>,
}

impl Atom for Udta {
    const KIND: FourCC = FourCC::new(b"udta");

    nested! {
        required: [ ],
        optional: [ Cprt, Meta, Kind, Rtng ],
        multiple: [ ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_udta_empty() {
        let expected = Udta {
            cprt: None,
            meta: None,
            kind: None,
            rtng: None,
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
            cprt: Some(Cprt {
                language: "und".into(),
                notice: "MIT or Apache".into(),
            }),
            meta: Some(Meta {
                hdlr: Hdlr {
                    handler: FourCC::new(b"fake"),
                    name: "".into(),
                },
                items: vec![],
            }),
            kind: Some(Kind {
                scheme_uri: "http://www.w3.org/TR/html5/".into(),
                value: "".into(),
            }),
            rtng: Some(Rtng {
                entity: b"BBFC".into(),
                criteria: b"PG13".into(),
                language: "eng".into(),
                rating_info: "test info".into(),
            }),
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
                cprt: Some(Cprt { language: "und".into(), notice: "ENST IsoMedia Conformance Files - ENST (c) 2006 - Rights released for ISO Conformance use".into() }),
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
                kind: Some(Kind {
                    scheme_uri: "urn:mpeg:dash:role:2011".into(),
                    value: "main".into()
                }),
                ..Default::default()
            }
        );

        let mut buf = Vec::new();
        udta.encode(&mut buf).unwrap();

        assert_eq!(buf, ENCODED_UDTA_WITH_KIND);
    }
}
