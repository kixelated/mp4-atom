use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Covr(pub Bytes);

impl Atom for Covr {
    const KIND: FourCC = FourCC::new(b"covr");

    fn decode_body(buf: &mut Bytes) -> Result<Self> {
        Ok(Covr(buf.decode()?))
    }

    fn encode_body(&self, buf: &mut BytesMut) -> Result<()> {
        buf.encode(&self.0)
    }
}
