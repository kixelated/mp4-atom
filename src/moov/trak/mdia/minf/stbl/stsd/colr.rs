use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Colr {
    Nclx {
        colour_primaries: u16,
        transfer_characteristics: u16,
        matrix_coefficients: u16,
        full_range_flag: bool,
    },
    Nclc {
        colour_primaries: u16,
        transfer_characteristics: u16,
        matrix_coefficients: u16,
    },
    Ricc {
        profile: Vec<u8>,
    },
    Prof {
        profile: Vec<u8>,
    },
}

impl Colr {
    pub fn new(
        colour_primaries: u16,
        transfer_characteristics: u16,
        matrix_coefficients: u16,
        full_range_flag: bool,
    ) -> Result<Self> {
        Ok(Colr::Nclx {
            colour_primaries,
            transfer_characteristics,
            matrix_coefficients,
            full_range_flag,
        })
    }
}

impl Atom for Colr {
    const KIND: FourCC = FourCC::new(b"colr");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        const NCLX: FourCC = FourCC::new(b"nclx");
        const NCLC: FourCC = FourCC::new(b"nclc");
        const PROF: FourCC = FourCC::new(b"prof");
        const RICC: FourCC = FourCC::new(b"rICC");

        let colour_type = FourCC::decode(buf)?;
        match colour_type {
            NCLX => {
                let colour_primaries = u16::decode(buf)?;
                let transfer_characteristics = u16::decode(buf)?;
                let matrix_coefficients = u16::decode(buf)?;
                let full_range_flag = u8::decode(buf)? == 0x80;
                Ok(Colr::Nclx {
                    colour_primaries,
                    transfer_characteristics,
                    matrix_coefficients,
                    full_range_flag,
                })
            }
            NCLC => {
                let colour_primaries = u16::decode(buf)?;
                let transfer_characteristics = u16::decode(buf)?;
                let matrix_coefficients = u16::decode(buf)?;
                Ok(Colr::Nclc {
                    colour_primaries,
                    transfer_characteristics,
                    matrix_coefficients,
                })
            }
            PROF => {
                let profile_len = buf.remaining();
                let profile = buf.slice(profile_len).to_vec();
                buf.advance(profile_len);
                Ok(Colr::Prof { profile })
            }
            RICC => {
                let profile_len = buf.remaining();
                let profile = buf.slice(profile_len).to_vec();
                buf.advance(profile_len);
                Ok(Colr::Ricc { profile })
            }
            _ => Err(Error::UnexpectedBox(colour_type)),
        }
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        match self {
            Colr::Nclx {
                colour_primaries,
                transfer_characteristics,
                matrix_coefficients,
                full_range_flag,
            } => {
                b"nclx".encode(buf)?;
                colour_primaries.encode(buf)?;
                transfer_characteristics.encode(buf)?;
                matrix_coefficients.encode(buf)?;
                if *full_range_flag {
                    0x80u8.encode(buf)?;
                } else {
                    0x00u8.encode(buf)?;
                }
            }
            Colr::Nclc {
                colour_primaries,
                transfer_characteristics,
                matrix_coefficients,
            } => {
                b"nclc".encode(buf)?;
                colour_primaries.encode(buf)?;
                transfer_characteristics.encode(buf)?;
                matrix_coefficients.encode(buf)?;
            }
            Colr::Ricc { profile } => {
                b"rICC".encode(buf)?;
                profile.encode(buf)?;
            }
            Colr::Prof { profile } => {
                b"prof".encode(buf)?;
                profile.encode(buf)?;
            }
        }
        Ok(())
    }
}

impl Default for Colr {
    fn default() -> Self {
        Colr::Nclx {
            // These match MIAF defaults (ISO/IEC 23000-22:2025 7.3.6.4), probably a reasonable set
            colour_primaries: 1,
            transfer_characteristics: 13,
            matrix_coefficients: 5,
            full_range_flag: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nclx_decode() {
        const ENCODED: &[u8] = &[
            0x00, 0x00, 0x00, 0x13, 0x63, 0x6f, 0x6c, 0x72, 0x6e, 0x63, 0x6c, 0x78, 0x00, 0x01,
            0x00, 0x01, 0x00, 0x01, 0x00,
        ];

        let buf = &mut std::io::Cursor::new(&ENCODED);

        let colr = Colr::decode(buf).expect("failed to decode colr");

        assert_eq!(
            colr,
            Colr::Nclx {
                colour_primaries: 1,
                transfer_characteristics: 1,
                matrix_coefficients: 1,
                full_range_flag: false
            }
        );
    }

    #[test]
    fn test_prof_decode_roundtrip() {
        // 19-byte colr: size=0x13, kind=colr, colour_type=prof, 7 bytes profile.
        // Pre-fix this failed with UnderDecode("colr") because the prof branch
        // borrowed the profile via buf.slice without advancing the cursor.
        const ENCODED: &[u8] = &[
            0x00, 0x00, 0x00, 0x13, b'c', b'o', b'l', b'r', b'p', b'r', b'o', b'f', 0x01, 0x02,
            0x03, 0x04, 0x05, 0x06, 0x07,
        ];
        let buf = &mut std::io::Cursor::new(&ENCODED);
        let colr = Colr::decode(buf).expect("failed to decode prof colr");
        assert_eq!(
            colr,
            Colr::Prof {
                profile: vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07],
            }
        );

        let mut out = Vec::new();
        colr.encode(&mut out).expect("encode prof colr");
        assert_eq!(out.as_slice(), ENCODED);
    }

    #[test]
    fn test_prof_does_not_leave_remaining_bytes_for_parent() {
        // Pin the strict-end-check contract: after decoding a prof colr, the
        // parent's remaining-bytes check must see an empty buffer. Pre-fix
        // this saw `profile_len` bytes still pending and returned
        // UnderDecode("colr") to the parent.
        const ENCODED: &[u8] = &[
            0x00, 0x00, 0x00, 0x10, b'c', b'o', b'l', b'r', b'p', b'r', b'o', b'f', 0xDE, 0xAD,
            0xBE, 0xEF,
        ];
        let buf = &mut std::io::Cursor::new(&ENCODED);
        let _ = Colr::decode(buf).expect("prof colr must decode without leaving trailing bytes");
        assert_eq!(
            <std::io::Cursor<&&[u8]> as crate::Buf>::remaining(buf),
            0,
            "parent end-check must see an empty body buffer"
        );
    }

    #[test]
    fn test_ricc_decode_roundtrip() {
        // Same shape with rICC colour_type instead of prof.
        const ENCODED: &[u8] = &[
            0x00, 0x00, 0x00, 0x13, b'c', b'o', b'l', b'r', b'r', b'I', b'C', b'C', 0xAA, 0xBB,
            0xCC, 0xDD, 0xEE, 0xFF, 0x00,
        ];
        let buf = &mut std::io::Cursor::new(&ENCODED);
        let colr = Colr::decode(buf).expect("failed to decode rICC colr");
        assert_eq!(
            colr,
            Colr::Ricc {
                profile: vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00],
            }
        );

        let mut out = Vec::new();
        colr.encode(&mut out).expect("encode rICC colr");
        assert_eq!(out.as_slice(), ENCODED);
    }

    #[test]
    fn test_nclx_encode() {
        const ENCODED: &[u8] = &[
            0x00, 0x00, 0x00, 0x13, 0x63, 0x6f, 0x6c, 0x72, 0x6e, 0x63, 0x6c, 0x78, 0x00, 0x01,
            0x00, 0x0d, 0x00, 0x06, 0x80,
        ];

        let colr = Colr::Nclx {
            colour_primaries: 1,
            transfer_characteristics: 13,
            matrix_coefficients: 6,
            full_range_flag: true,
        };

        let mut buf = Vec::new();
        colr.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED);
    }
}
