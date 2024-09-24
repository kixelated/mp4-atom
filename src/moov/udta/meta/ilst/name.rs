use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Name(pub String);

impl Atom for Name {
    const KIND: FourCC = FourCC::new(b"name");

    fn decode_body(buf: &mut Bytes) -> Result<Self> {
        Ok(Name(buf.decode()?))
    }

    fn encode_body(&self, buf: &mut BytesMut) -> Result<()> {
        self.0.as_str().encode(buf)
    }
}
