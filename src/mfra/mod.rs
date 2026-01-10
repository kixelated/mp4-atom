mod mfro;
mod tfra;

pub use mfro::*;
pub use tfra::*;

use crate::*;

/// MovieFragmentRandomAccessBox (`mfra`).
///
/// See ISO/IEC 14496-12:2022 section 8.8.9
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mfra {
    pub tfra: Vec<Tfra>,
    pub mfro: Mfro,
}

// We can't use the normal nested macro here, because we need mfro to be written last
impl Atom for Mfra {
    const KIND: FourCC = FourCC::new(b"mfra");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let mut mfro = None;
        let mut tfra = Vec::new();

        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Mfro(atom) => {
                    if mfro.is_some() {
                        return Err(Error::DuplicateBox(Mfro::KIND));
                    }
                    mfro = Some(atom);
                }
                Any::Tfra(atom) => {
                    tfra.push(atom);
                }
                Any::Skip(atom) => tracing::debug!(size = atom.zeroed.size, "skipping skip box"),
                Any::Free(atom) => tracing::debug!(size = atom.zeroed.size, "skipping free box"),
                unknown => Self::decode_unknown(&unknown)?,
            }
        }
        Ok(Self {
            mfro: mfro.ok_or(Error::MissingBox(Mfro::KIND))?,
            tfra,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.tfra.iter().try_for_each(|x| x.encode(buf))?;
        self.mfro.encode(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // From MPEG File Format Conformance suite: fragment_random_access-2.mp4
    const ENCODED_MFRA: &[u8] = &[
        0x00, 0x00, 0x00, 0xa9, 0x6d, 0x66, 0x72, 0x61, 0x00, 0x00, 0x00, 0x91, 0x74, 0x66, 0x72,
        0x61, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x0b, 0x00, 0x06, 0xdd, 0xd0, 0x00, 0x00, 0xeb, 0x1b, 0x01, 0x01, 0x01, 0x00, 0x08,
        0x3d, 0x60, 0x00, 0x00, 0xeb, 0x1b, 0x01, 0x02, 0x01, 0x00, 0x09, 0x9c, 0xf0, 0x00, 0x00,
        0xeb, 0x1b, 0x01, 0x03, 0x01, 0x00, 0x0a, 0xfc, 0x80, 0x00, 0x00, 0xeb, 0x1b, 0x01, 0x04,
        0x01, 0x00, 0x0c, 0x5c, 0x10, 0x00, 0x00, 0xeb, 0x1b, 0x01, 0x05, 0x01, 0x00, 0x0d, 0xbb,
        0xa0, 0x00, 0x01, 0xdc, 0xa3, 0x01, 0x01, 0x01, 0x00, 0x0f, 0x1b, 0x30, 0x00, 0x01, 0xdc,
        0xa3, 0x01, 0x02, 0x01, 0x00, 0x10, 0x7a, 0xc0, 0x00, 0x01, 0xdc, 0xa3, 0x01, 0x03, 0x01,
        0x00, 0x11, 0xda, 0x50, 0x00, 0x01, 0xdc, 0xa3, 0x01, 0x04, 0x01, 0x00, 0x13, 0x39, 0xe0,
        0x00, 0x01, 0xdc, 0xa3, 0x01, 0x05, 0x01, 0x00, 0x14, 0x99, 0x70, 0x00, 0x02, 0xc4, 0x33,
        0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x10, 0x6d, 0x66, 0x72, 0x6f, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0xa9,
    ];

    fn get_reference_mfra() -> Mfra {
        Mfra {
            tfra: vec![Tfra {
                track_id: 1,
                entries: vec![
                    FragmentInfo {
                        time: 450000,
                        moof_offset: 60187,
                        traf_number: 1,
                        trun_number: 1,
                        sample_delta: 1,
                    },
                    FragmentInfo {
                        time: 540000,
                        moof_offset: 60187,
                        traf_number: 1,
                        trun_number: 2,
                        sample_delta: 1,
                    },
                    FragmentInfo {
                        time: 630000,
                        moof_offset: 60187,
                        traf_number: 1,
                        trun_number: 3,
                        sample_delta: 1,
                    },
                    FragmentInfo {
                        time: 720000,
                        moof_offset: 60187,
                        traf_number: 1,
                        trun_number: 4,
                        sample_delta: 1,
                    },
                    FragmentInfo {
                        time: 810000,
                        moof_offset: 60187,
                        traf_number: 1,
                        trun_number: 5,
                        sample_delta: 1,
                    },
                    FragmentInfo {
                        time: 900000,
                        moof_offset: 122019,
                        traf_number: 1,
                        trun_number: 1,
                        sample_delta: 1,
                    },
                    FragmentInfo {
                        time: 990000,
                        moof_offset: 122019,
                        traf_number: 1,
                        trun_number: 2,
                        sample_delta: 1,
                    },
                    FragmentInfo {
                        time: 1080000,
                        moof_offset: 122019,
                        traf_number: 1,
                        trun_number: 3,
                        sample_delta: 1,
                    },
                    FragmentInfo {
                        time: 1170000,
                        moof_offset: 122019,
                        traf_number: 1,
                        trun_number: 4,
                        sample_delta: 1,
                    },
                    FragmentInfo {
                        time: 1260000,
                        moof_offset: 122019,
                        traf_number: 1,
                        trun_number: 5,
                        sample_delta: 1,
                    },
                    FragmentInfo {
                        time: 1350000,
                        moof_offset: 181299,
                        traf_number: 1,
                        trun_number: 1,
                        sample_delta: 1,
                    },
                ],
            }],
            mfro: Mfro { parent_size: 169 },
        }
    }

    #[test]
    fn test_mfra_decode() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_MFRA);
        let mfra = Mfra::decode(buf).expect("failed to decode mfra");
        assert_eq!(mfra, get_reference_mfra());
    }

    #[test]
    fn test_mfra_encode() {
        let mfra = get_reference_mfra();
        let mut buf = Vec::new();
        mfra.encode(&mut buf).unwrap();
        assert_eq!(buf, ENCODED_MFRA);
    }
}
