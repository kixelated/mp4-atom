use crate::*;

/// I would not recommend using this for large files, as it reads the entire file into memory.
/// Instead, use [ReadFrom] to read the [Header] first followed by the mdat data.
#[derive(Debug, Clone, PartialEq)]
pub struct Mdat {
    pub data: Bytes,
}

impl Atom for Mdat {
    const KIND: FourCC = FourCC::new(b"mdat");

    fn decode_atom(buf: &mut Bytes) -> Result<Self> {
        Ok(Mdat {
            data: buf.decode()?,
        })
    }

    fn encode_atom(&self, buf: &mut BytesMut) -> Result<()> {
        buf.encode(&self.data)
    }
}
