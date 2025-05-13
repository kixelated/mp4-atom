use crate::*;

/// ItemDataBox
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Idat {
    pub data: Vec<u8>,
}

impl Atom for Idat {
    const KIND: FourCC = FourCC::new(b"idat");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Idat {
            data: Vec::decode(buf)?,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.data.encode(buf)
    }
}
