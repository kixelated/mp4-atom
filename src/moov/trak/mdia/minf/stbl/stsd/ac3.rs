use crate::*;

// See ETSI TS 102 366 V1.4.1 (2017-09) for details of AC-3 and EAC-3

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ac3 {
    pub audio: Audio,
    pub dac3: Ac3SpecificBox,
    #[cfg(feature = "fault-tolerant")]
    pub unexpected: Vec<Any>,
}

impl Atom for Ac3 {
    const KIND: FourCC = FourCC::new(b"ac-3");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let audio = Audio::decode(buf)?;

        let mut dac3 = None;
        #[cfg(feature = "fault-tolerant")]
        let mut unexpected = Vec::new();

        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Ac3SpecificBox(atom) => dac3 = atom.into(),
                _ => {
                    tracing::warn!("unknown atom: {:?}", atom);
                    #[cfg(feature = "fault-tolerant")]
                    unexpected.push(atom);
                }
            }
        }

        Ok(Self {
            audio,
            dac3: dac3.ok_or(Error::MissingBox(Ac3SpecificBox::KIND))?,
            #[cfg(feature = "fault-tolerant")]
            unexpected,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.audio.encode(buf)?;
        self.dac3.encode(buf)?;
        Ok(())
    }
}

// AC-3 specific data
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ac3SpecificBox {
    pub fscod: u8,
    pub bsid: u8,
    pub bsmod: u8,
    pub acmod: u8,
    pub lfeon: bool,
    pub bit_rate_code: u8,
}

impl Atom for Ac3SpecificBox {
    const KIND: FourCC = FourCC::new(b"dac3");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let body_bytes = u24::decode(buf)?;
        let body: u32 = body_bytes.into();
        let fscod = ((body >> 22) & 0b11) as u8;
        let bsid = ((body >> 17) & 0b11111) as u8;
        let bsmod = ((body >> 14) & 0b111) as u8;
        let acmod = ((body >> 11) & 0b111) as u8;
        let lfeon = ((body >> 10) & 0b1) == 0b1;
        let bit_rate_code = ((body >> 5) & 0b11111) as u8;
        Ok(Self {
            fscod,
            bsid,
            bsmod,
            acmod,
            lfeon,
            bit_rate_code,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let body: u32 = ((self.bit_rate_code as u32) << 5)
            | ((self.acmod as u32) << 11)
            | ((self.bsmod as u32) << 14)
            | ((self.bsid as u32) << 17)
            | ((self.fscod as u32) << 22)
            | (if self.lfeon { 0x1u32 << 10 } else { 0u32 });
        let body_bytes: u24 = body
            .try_into()
            .map_err(|_| Error::TooLarge(Ac3SpecificBox::KIND))?;
        body_bytes.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Streaminfo metadata block only
    const ENCODED_AC3: &[u8] = &[
        0x00, 0x00, 0x00, 0x2f, 0x61, 0x63, 0x2d, 0x33, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x10, 0x00, 0x00,
        0x00, 0x00, 0xac, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0b, 0x64, 0x61, 0x63, 0x33, 0x50,
        0x11, 0x40,
    ];

    #[test]
    fn test_ac3_decode() {
        let buf: &mut std::io::Cursor<&[u8]> = &mut std::io::Cursor::new(ENCODED_AC3);

        let ac3 = Ac3::decode(buf).expect("failed to decode ac-3");

        assert_eq!(
            ac3,
            Ac3 {
                audio: Audio {
                    data_reference_index: 1,
                    channel_count: 2,
                    sample_size: 16,
                    sample_rate: 44100.into()
                },
                dac3: Ac3SpecificBox {
                    fscod: 1,
                    bsid: 8,
                    bsmod: 0,
                    acmod: 2,
                    lfeon: false,
                    bit_rate_code: 10
                },
                #[cfg(feature = "fault-tolerant")]
                unexpected: vec![],
            }
        );
    }

    #[test]
    fn test_ac3_encode() {
        let ac3 = Ac3 {
            audio: Audio {
                data_reference_index: 1,
                channel_count: 2,
                sample_size: 16,
                sample_rate: 44100.into(),
            },
            dac3: Ac3SpecificBox {
                fscod: 1,
                bsid: 8,
                bsmod: 0,
                acmod: 2,
                lfeon: false,
                bit_rate_code: 10,
            },
            #[cfg(feature = "fault-tolerant")]
            unexpected: vec![],
        };

        let mut buf = Vec::new();
        ac3.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_AC3);
    }
}
