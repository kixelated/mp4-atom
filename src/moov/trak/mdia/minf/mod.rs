mod dinf;
mod smhd;
mod stbl;
mod vmhd;

pub use dinf::*;
pub use smhd::*;
pub use stbl::*;
pub use vmhd::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Minf {
    pub vmhd: Option<Vmhd>,
    pub smhd: Option<Smhd>,
    pub dinf: Dinf,
    pub stbl: Stbl,
    pub hdlr: Option<Hdlr>,
}

impl Atom for Minf {
    const KIND: FourCC = FourCC::new(b"minf");

    nested! {
        required: [ Dinf, Stbl ],
        optional: [ Vmhd, Smhd, Hdlr ],
        multiple: [],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minf_with_hdlr() {
        // Test case with hdlr atom in minf
        let minf = Minf {
            vmhd: Some(Vmhd {
                graphics_mode: 0,
                op_color: RgbColor {
                    red: 0,
                    green: 0,
                    blue: 0,
                },
            }),
            smhd: None,
            dinf: Dinf {
                dref: Dref {
                    urls: vec![Url {
                        location: "".into(),
                    }],
                },
            },
            stbl: Stbl {
                stsd: Stsd { codecs: vec![] },
                stts: Stts { entries: vec![] },
                stsc: Stsc { entries: vec![] },
                stsz: Stsz {
                    samples: StszSamples::Identical { size: 0, count: 0 },
                },
                stco: None,
                co64: None,
                stss: None,
                ctts: None,
                sbgp: vec![],
                sgpd: vec![],
                subs: vec![],
                saiz: vec![],
                saio: vec![],
            },
            hdlr: Some(Hdlr {
                handler: b"vide".into(),
                name: "VideoHandler".into(),
            }),
        };

        // Encode
        let mut buf = Vec::new();
        minf.encode(&mut buf).unwrap();

        // Decode and verify round-trip
        let mut cursor = std::io::Cursor::new(&buf);
        let decoded = Minf::decode(&mut cursor).expect("failed to decode minf with hdlr");

        // Note: stsz with Identical { size: 0, count: 0 } encodes to Different { sizes: [] }
        // when size is 0, so we need to adjust expectation
        let expected = Minf {
            stbl: Stbl {
                stsz: Stsz {
                    samples: StszSamples::Different { sizes: vec![] },
                },
                ..minf.stbl.clone()
            },
            ..minf.clone()
        };

        assert_eq!(decoded, expected);
        assert!(decoded.hdlr.is_some());
        assert_eq!(decoded.hdlr.as_ref().unwrap().handler, FourCC::new(b"vide"));
        assert_eq!(decoded.hdlr.as_ref().unwrap().name, "VideoHandler");
    }

    #[test]
    fn test_minf_without_hdlr() {
        // Test case without hdlr atom in minf (traditional structure)
        let minf = Minf {
            vmhd: Some(Vmhd {
                graphics_mode: 0,
                op_color: RgbColor {
                    red: 0,
                    green: 0,
                    blue: 0,
                },
            }),
            smhd: None,
            dinf: Dinf {
                dref: Dref {
                    urls: vec![Url {
                        location: "".into(),
                    }],
                },
            },
            stbl: Stbl {
                stsd: Stsd { codecs: vec![] },
                stts: Stts { entries: vec![] },
                stsc: Stsc { entries: vec![] },
                stsz: Stsz {
                    samples: StszSamples::Identical { size: 0, count: 0 },
                },
                stco: None,
                co64: None,
                stss: None,
                ctts: None,
                sbgp: vec![],
                sgpd: vec![],
                subs: vec![],
                saiz: vec![],
                saio: vec![],
            },
            hdlr: None,
        };

        // Encode
        let mut buf = Vec::new();
        minf.encode(&mut buf).unwrap();

        // Decode and verify round-trip
        let mut cursor = std::io::Cursor::new(&buf);
        let decoded = Minf::decode(&mut cursor).expect("failed to decode minf without hdlr");

        // Note: stsz with Identical { size: 0, count: 0 } encodes to Different { sizes: [] }
        // when size is 0, so we need to adjust expectation
        let expected = Minf {
            stbl: Stbl {
                stsz: Stsz {
                    samples: StszSamples::Different { sizes: vec![] },
                },
                ..minf.stbl.clone()
            },
            ..minf.clone()
        };

        assert_eq!(decoded, expected);
        assert!(decoded.hdlr.is_none());
    }
}
