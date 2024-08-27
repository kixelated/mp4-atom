use crate::*;

// A helper to encode/decode a known atom type.
pub trait Atom: Sized {
    const KIND: FourCC;

    fn decode_atom(buf: &mut Buf) -> Result<Self>;
    fn encode_atom(&self, buf: &mut BufMut) -> Result<()>;
}

impl<T: Atom> Encode for T {
    fn encode(&self, buf: &mut BufMut) -> Result<()> {
        let start = buf.len();

        // Encode a 0 for the size, we'll come back to it later
        0u32.encode(buf)?;
        Self::KIND.encode(buf)?;
        self.encode_atom(buf)?;

        // Update the size field
        // TODO support sizes larger than u32 (4GB)
        let size = (buf.len() - start)
            .try_into()
            .map_err(|_| Error::LongWrite)?;

        buf.u32_at(size, start);
        Ok(())
    }
}

impl<T: Atom> Decode for T {
    fn decode(buf: &mut Buf) -> Result<Self> {
        let header = Header::decode(buf)?;
        let buf = &mut buf.take(header.size.unwrap_or(buf.len()))?;
        Ok(Self::decode_atom(buf)?)
    }
}
