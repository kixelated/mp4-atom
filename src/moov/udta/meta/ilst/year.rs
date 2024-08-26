use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Year(pub String);

impl Atom for Year {
    const KIND: FourCC = FourCC::new(b"day ");

    fn decode_atom(buf: &mut Bytes) -> Result<Self> {
        Ok(Self(buf.decode()?))
    }

    fn encode_atom(&self, buf: &mut BytesMut) -> Result<()> {
        self.0.as_str().encode(buf)
    }
}
