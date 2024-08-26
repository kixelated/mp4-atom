use crate::*;

// I would not recommend using this for large files, as it reads the entire file into memory.
// Instead, use MdatChunked (TODO)
#[derive(Debug, Clone, PartialEq)]
pub struct Mdat {
    pub data: Bytes,
}

impl Atom for Mdat {
    const KIND: FourCC = FourCC::new(b"mdat");

    fn decode_inner<B: Buf>(mut buf: &mut B) -> Result<Self> {
        Ok(Mdat {
            data: buf.decode()?,
        })
    }

    fn encode_inner<B: BufMut>(&self, mut buf: &mut B) -> Result<()> {
        buf.encode(&self.data)
    }

    fn encode_inner_size(&self) -> usize {
        self.data.encode_size()
    }
}
