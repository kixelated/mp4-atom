use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Covr(pub Bytes);

impl Atom for Covr {
    const KIND: FourCC = FourCC::new(b"covr");

    fn decode_atom<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Covr(buf.decode()?))
    }

    fn encode_atom(&self, buf: &mut BytesMut) -> Result<()> {
        buf.encode(&self.0)
    }
}
