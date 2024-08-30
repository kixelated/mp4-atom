use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name(pub String);

impl Atom for Name {
    const KIND: FourCC = FourCC::new(b"name");

    fn decode_atom<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Name(buf.decode()?))
    }

    fn encode_atom(&self, buf: &mut BytesMut) -> Result<()> {
        self.0.as_str().encode(buf)
    }
}
