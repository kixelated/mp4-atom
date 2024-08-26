use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Unknown {
    pub kind: FourCC,
    pub data: Bytes,
}

impl Encode for Unknown {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let header = Header {
            kind: self.kind,
            size: Some(self.data.len()),
        };

        header.encode(buf)?;
        self.data.encode(buf)
    }

    fn encode_size(&self) -> usize {
        let header = Header {
            kind: self.kind,
            size: Some(self.data.len()),
        };

        header.encode_size() + self.data.encode_size()
    }
}

impl Decode for Unknown {
    fn decode<B: Buf>(mut buf: &mut B) -> Result<Self> {
        let header = Header::decode(&mut buf)?;
        let mut buf = &mut buf.take(header.size.unwrap_or(buf.remaining()));

        Ok(Unknown {
            kind: header.kind,
            data: buf.decode()?,
        })
    }
}
