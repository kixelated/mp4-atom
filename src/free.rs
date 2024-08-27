use crate::*;

pub struct Free {
    pub size: usize,
}

impl Atom for Free {
    const KIND: FourCC = FourCC::new(b"free");

    fn decode_atom(buf: &mut Buf) -> Result<Self> {
        let size = buf.size();
        buf.rest();
        Ok(Free { size })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        buf.zero(self.size)?;
        Ok(())
    }
}
