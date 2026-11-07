use crate::*;

/// OriginalFormatBox (`frma`), ISO/IEC 14496-12:2026 Section 13.4.3: the type of the
/// sample entry a transformation replaced. It sits in a `sinf` (protection), `rinf`
/// (restriction) or `cinf` (incomplete track); this crate handles `sinf`.
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
