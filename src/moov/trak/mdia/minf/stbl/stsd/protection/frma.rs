use crate::*;

/// OriginalFormatBox (`frma`), ISO/IEC 14496-12:2022 Sect 8.12.3:
/// the sample entry type a protected entry (`encv`, `enca`) replaced.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Frma {
    pub data_format: FourCC,
}

impl Atom for Frma {
    const KIND: FourCC = FourCC::new(b"frma");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Frma {
            data_format: FourCC::decode(buf)?,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.data_format.encode(buf)
    }
}
