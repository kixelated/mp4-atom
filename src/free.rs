use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Free {
    pub size: usize,
}

impl Atom for Free {
    const KIND: FourCC = FourCC::new(b"free");

    fn decode_atom(buf: &mut Bytes) -> Result<Self> {
        let data = buf.split_to(buf.len());
        Ok(Free { size: data.len() })
    }

    fn encode_atom(&self, buf: &mut BytesMut) -> Result<()> {
        buf.put_bytes(0, self.size);
        Ok(())
    }
}
