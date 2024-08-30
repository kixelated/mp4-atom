use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Free {
    pub size: usize,
}

impl Atom for Free {
    const KIND: FourCC = FourCC::new(b"free");

    fn decode_atom<B: Buf>(buf: &mut B) -> Result<Self> {
        let size = buf.remaining();
        buf.advance(size);
        Ok(Free { size })
    }

    fn encode_atom(&self, buf: &mut BytesMut) -> Result<()> {
        buf.put_bytes(0, self.size);
        Ok(())
    }
}
