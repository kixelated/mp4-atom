use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Year(pub String);

impl Atom for Year {
    const KIND: FourCC = FourCC::new(b"day ");

    fn decode_body(buf: &mut Bytes) -> Result<Self> {
        Ok(Self(buf.decode()?))
    }

    fn encode_body(&self, buf: &mut BytesMut) -> Result<()> {
        self.0.as_str().encode(buf)
    }
}
