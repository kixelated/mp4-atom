use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Desc(pub String);

impl Atom for Desc {
    const KIND: FourCC = FourCC::new(b"desc");

    fn decode_body(buf: &mut Bytes) -> Result<Self> {
        Ok(Desc(buf.decode()?))
    }

    fn encode_body(&self, buf: &mut BytesMut) -> Result<()> {
        self.0.as_str().encode(buf)
    }
}
