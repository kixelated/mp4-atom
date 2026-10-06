use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Damr {
    pub vendor: FourCC,
    pub decoder_version: u8,
    pub mode_set: u16,
    pub mode_change_period: u8,
    pub frames_per_sample: u8,
}

impl Atom for Damr {
    const KIND: FourCC = FourCC::new(b"damr");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let vendor = FourCC::decode(buf)?;
        let decoder_version = u8::decode(buf)?;
        let mode_set = u16::decode(buf)?;
        let mode_change_period = u8::decode(buf)?;
        let frames_per_sample = u8::decode(buf)?;
        Ok(Damr {
            vendor,
            decoder_version,
            mode_set,
            mode_change_period,
            frames_per_sample,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.vendor.encode(buf)?;
        self.decoder_version.encode(buf)?;
        self.mode_set.encode(buf)?;
        self.mode_change_period.encode(buf)?;
        self.frames_per_sample.encode(buf)?;
        Ok(())
    }
}
