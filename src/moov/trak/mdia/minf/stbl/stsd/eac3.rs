use crate::*;

// See ETSI TS 102 366 V1.4.1 (2017-09) for details of AC-3 and EAC-3

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Eac3 {
    pub audio: Audio,
    pub dec3: Ec3SpecificBox,
}

impl Atom for Eac3 {
    const KIND: FourCC = FourCC::new(b"ec-3");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let audio = Audio::decode(buf)?;

        let mut dec3 = None;

        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Ec3SpecificBox(atom) => dec3 = atom.into(),
                _ => tracing::warn!("unknown atom: {:?}", atom),
            }
        }

        Ok(Self {
            audio,
            dec3: dec3.ok_or(Error::MissingBox(Ec3SpecificBox::KIND))?,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.audio.encode(buf)?;
        self.dec3.encode(buf)?;
        Ok(())
    }
}

// EAC-3 specific data
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ec3IndependentSubstream {
    pub fscod: u8,
    pub bsid: u8,
    pub asvc: bool,
    pub bsmod: u8,
    pub acmod: u8,
    pub lfeon: bool,
    pub num_dep_sub: u8,
    pub chan_loc: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ec3SpecificBox {
    pub data_rate: u16,
    pub substreams: Vec<Ec3IndependentSubstream>,
}

impl Atom for Ec3SpecificBox {
    const KIND: FourCC = FourCC::new(b"dec3");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let header = u16::decode(buf)?;
        let data_rate = header >> 3;
        let num_ind_sub = header & 0b111;
        let mut substreams = Vec::with_capacity(num_ind_sub as usize + 1);
        for _ in 0..num_ind_sub + 1 {
            let b0 = u8::decode(buf)?;
            let fscod = b0 >> 6;
            let bsid = (b0 >> 1) & 0b11111;
            // ignore low bit - reserved
            let b1 = u8::decode(buf)?;
            let asvc = (b1 & 0x80) == 0x80;
            let bsmod = (b1 >> 4) & 0b111;
            let acmod = (b1 >> 1) & 0b111;
            let lfeon = (b1 & 0b1) == 0b1;
            let b2 = u8::decode(buf)?;
            // ignore next three bits
            let num_dep_sub = (b2 >> 1) & 0b1111;
            if num_dep_sub > 0 {
                let b3 = u8::decode(buf)? as u16;
                let chan_loc = ((b2 as u16 & 0x01) << 8) | b3;
                substreams.push(Ec3IndependentSubstream {
                    fscod,
                    bsid,
                    asvc,
                    bsmod,
                    acmod,
                    lfeon,
                    num_dep_sub,
                    chan_loc: Some(chan_loc),
                });
            } else {
                // ignore last bit in b2
                substreams.push(Ec3IndependentSubstream {
                    fscod,
                    bsid,
                    asvc,
                    bsmod,
                    acmod,
                    lfeon,
                    num_dep_sub,
                    chan_loc: None,
                });
            }
        }
        Ok(Self {
            data_rate,
            substreams,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let header = self.data_rate << 3 | self.substreams.len().saturating_sub(1) as u16;
        header.encode(buf)?;
        for substream in &self.substreams {
            // low bit is reserved = 0
            let b = (substream.fscod << 6) | (substream.bsid << 1);
            b.encode(buf)?;
            let b = (if substream.asvc { 0x80 } else { 0x00 })
                | (substream.bsmod << 4)
                | (substream.acmod << 1)
                | (if substream.lfeon { 0x01 } else { 0x00 });
            b.encode(buf)?;
            if substream.num_dep_sub > 0 {
                let b: u16 =
                    ((substream.num_dep_sub as u16) << 9) | (substream.chan_loc.unwrap_or(0u16));
                b.encode(buf)?;
            } else {
                // high and low bits are reserved = 0
                let b = substream.num_dep_sub << 1;
                b.encode(buf)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // EAC-3 metadata block only
    const ENCODED_EAC3: &[u8] = &[
        0x00, 0x00, 0x00, 0x31, 0x65, 0x63, 0x2d, 0x33, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x10, 0x00, 0x00,
        0x00, 0x00, 0xac, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0d, 0x64, 0x65, 0x63, 0x33, 0x5f,
        0xc0, 0x60, 0x04, 0x00,
    ];

    #[test]
    fn test_eac3_decode() {
        let buf: &mut std::io::Cursor<&[u8]> = &mut std::io::Cursor::new(ENCODED_EAC3);

        let eac3 = Eac3::decode(buf).expect("failed to decode eac-3");

        assert_eq!(
            eac3,
            Eac3 {
                audio: Audio {
                    data_reference_index: 1,
                    channel_count: 2,
                    sample_size: 16,
                    sample_rate: 44100.into()
                },
                dec3: Ec3SpecificBox {
                    data_rate: 3064,
                    substreams: vec![Ec3IndependentSubstream {
                        fscod: 1,
                        bsid: 16,
                        asvc: false,
                        bsmod: 0,
                        acmod: 2,
                        lfeon: false,
                        num_dep_sub: 0,
                        chan_loc: None
                    }]
                }
            }
        );
    }

    #[test]
    fn test_eac3_encode() {
        let eac3 = Eac3 {
            audio: Audio {
                data_reference_index: 1,
                channel_count: 2,
                sample_size: 16,
                sample_rate: 44100.into(),
            },
            dec3: Ec3SpecificBox {
                data_rate: 3064,
                substreams: vec![Ec3IndependentSubstream {
                    fscod: 1,
                    bsid: 16,
                    asvc: false,
                    bsmod: 0,
                    acmod: 2,
                    lfeon: false,
                    num_dep_sub: 0,
                    chan_loc: None,
                }],
            },
        };

        let mut buf = Vec::new();
        eac3.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_EAC3);
    }

    #[test]
    fn test_eac3_with_dependent_substreams() {
        // Test case with dependent substreams (num_dep_sub > 0)
        let eac3 = Eac3 {
            audio: Audio {
                data_reference_index: 1,
                channel_count: 6,
                sample_size: 16,
                sample_rate: 48000.into(),
            },
            dec3: Ec3SpecificBox {
                data_rate: 768,
                substreams: vec![Ec3IndependentSubstream {
                    fscod: 0, // 48 kHz
                    bsid: 16,
                    asvc: true,
                    bsmod: 1,
                    acmod: 7, // 3/2 (L, C, R, Ls, Rs)
                    lfeon: true,
                    num_dep_sub: 2,
                    chan_loc: Some(0x1FF),
                }],
            },
        };

        // Encode
        let mut buf = Vec::new();
        eac3.encode(&mut buf).unwrap();

        // Decode and verify round-trip
        let mut cursor = std::io::Cursor::new(&buf);
        let decoded = Eac3::decode(&mut cursor).expect("failed to decode eac-3");

        assert_eq!(decoded, eac3);
        assert_eq!(decoded.dec3.substreams[0].chan_loc, Some(0x1FF));
    }

    #[test]
    fn test_eac3_multiple_substreams() {
        // Test case with multiple independent substreams
        let eac3 = Eac3 {
            audio: Audio {
                data_reference_index: 1,
                channel_count: 8,
                sample_size: 16,
                sample_rate: 48000.into(),
            },
            dec3: Ec3SpecificBox {
                data_rate: 1536,
                substreams: vec![
                    Ec3IndependentSubstream {
                        fscod: 0, // 48 kHz
                        bsid: 16,
                        asvc: false,
                        bsmod: 0,
                        acmod: 7, // 3/2
                        lfeon: true,
                        num_dep_sub: 0,
                        chan_loc: None,
                    },
                    Ec3IndependentSubstream {
                        fscod: 0, // 48 kHz
                        bsid: 16,
                        asvc: false,
                        bsmod: 0,
                        acmod: 2, // 2/0 (L, R)
                        lfeon: false,
                        num_dep_sub: 0,
                        chan_loc: None,
                    },
                ],
            },
        };

        // Encode
        let mut buf = Vec::new();
        eac3.encode(&mut buf).unwrap();

        // Decode and verify round-trip
        let mut cursor = std::io::Cursor::new(&buf);
        let decoded = Eac3::decode(&mut cursor).expect("failed to decode eac-3");

        assert_eq!(decoded, eac3);
        assert_eq!(decoded.dec3.substreams.len(), 2);
    }

    #[test]
    fn test_eac3_with_reserved_bits() {
        // Test case with reserved bits following the substream data
        // This simulates a dec3 box with extra padding/reserved bits at the end
        let encoded_with_reserved: &[u8] = &[
            0x00, 0x00, 0x00, 0x33, 0x65, 0x63, 0x2d, 0x33, // ec-3 atom header (size=51)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // audio sample entry
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // audio sample entry
            0x00, 0x02, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, // audio sample entry
            0xac, 0x44, 0x00, 0x00, // audio sample entry
            0x00, 0x00, 0x00, 0x0f, 0x64, 0x65, 0x63, 0x33, // dec3 header (size 15 bytes)
            0x5f, 0xc0, // header: data_rate=3064, num_ind_sub=0
            0x60, 0x04, 0x00, // substream data
            0xAB, 0xCD, // extra reserved bits (2 bytes)
        ];

        let mut cursor = std::io::Cursor::new(encoded_with_reserved);
        let decoded = Eac3::decode(&mut cursor).expect("failed to decode eac-3 with reserved bits");

        // Should decode successfully and ignore the reserved bits
        assert_eq!(
            decoded,
            Eac3 {
                audio: Audio {
                    data_reference_index: 1,
                    channel_count: 2,
                    sample_size: 16,
                    sample_rate: 44100.into()
                },
                dec3: Ec3SpecificBox {
                    data_rate: 3064,
                    substreams: vec![Ec3IndependentSubstream {
                        fscod: 1,
                        bsid: 16,
                        asvc: false,
                        bsmod: 0,
                        acmod: 2,
                        lfeon: false,
                        num_dep_sub: 0,
                        chan_loc: None
                    }]
                }
            }
        );
    }
}
