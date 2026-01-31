use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Samr {
    pub amrsampleentry: AmrSampleEntry,
    pub damr: Damr,
}

impl Atom for Samr {
    const KIND: FourCC = FourCC::new(b"samr");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let amrsampleentry = AmrSampleEntry::decode(buf)?;

        let mut damr = None;
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Damr(atom) => damr = atom.into(),
                unknown => Self::decode_unknown(&unknown)?,
            }
        }

        Ok(Self {
            amrsampleentry,
            damr: damr.ok_or(Error::MissingBox(Damr::KIND))?,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.amrsampleentry.encode(buf)?;
        self.damr.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENCODED_SAMR: &[u8] = &[
        0x00, 0x00, 0x00, 0x35, 0x73, 0x61, 0x6d, 0x72, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x10, 0x00, 0x00,
        0x00, 0x00, 0x1f, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0x64, 0x61, 0x6d, 0x72, 0x65,
        0x72, 0x61, 0x74, 0x00, 0x00, 0x80, 0x00, 0x01,
    ];

    #[test]
    fn test_samr_decode() {
        let buf = &mut std::io::Cursor::new(&ENCODED_SAMR);

        let samr = Samr::decode(buf).expect("failed to decode samr");

        assert_eq!(
            samr,
            Samr {
                amrsampleentry: AmrSampleEntry {
                    data_reference_index: 1,
                    timescale: 8000
                },
                damr: Damr {
                    vendor: b"erat".into(),
                    decoder_version: 0,
                    mode_set: 128,
                    mode_change_period: 0,
                    frames_per_sample: 1
                }
            }
        );
    }

    #[test]
    fn test_samr_encode() {
        let samr = Samr {
            amrsampleentry: AmrSampleEntry {
                data_reference_index: 1,
                timescale: 8000,
            },
            damr: Damr {
                vendor: b"erat".into(),
                decoder_version: 0,
                mode_set: 128,
                mode_change_period: 0,
                frames_per_sample: 1,
            },
        };

        let mut buf = Vec::new();
        samr.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_SAMR);
    }
}
