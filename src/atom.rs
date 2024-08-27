use crate::*;

// A helper to encode/decode a known atom type.
pub trait Atom: Sized {
    const KIND: FourCC;

    fn decode_atom(buf: &mut Bytes) -> Result<Self>;
    fn encode_atom(&self, buf: &mut BytesMut) -> Result<()>;
}

impl<T: Atom> Encode for T {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        let start = buf.len();

        // Encode a 0 for the size, we'll come back to it later
        0u32.encode(buf)?;
        Self::KIND.encode(buf)?;
        self.encode_atom(buf)?;

        // Update the size field
        // TODO support sizes larger than u32 (4GB)
        let size: u32 = (buf.len() - start)
            .try_into()
            .map_err(|_| Error::LongWrite)?;

        buf[start..start + 4].copy_from_slice(&size.to_be_bytes());

        Ok(())
    }
}

impl<T: Atom> Decode for T {
    fn decode(buf: &mut Bytes) -> Result<Self> {
        let header = Header::decode(buf)?;
        let size = header.size.unwrap_or(buf.len());
        if buf.len() < size {
            return Err(Error::ShortRead);
        }

        let mut data = buf.split_to(size);
        let atom = Self::decode_atom(&mut data)?;
        if !data.is_empty() {
            return Err(Error::ShortRead);
        }

        Ok(atom)
    }
}
