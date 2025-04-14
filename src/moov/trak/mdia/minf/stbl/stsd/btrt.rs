use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Btrt {
    pub buffer_size_db: u32,
    pub max_bitrate: u32,
    pub avg_bitrate: u32,
}

impl Btrt {
    pub fn new(buffer_size_db: u32, max_bitrate: u32, avg_bitrate: u32) -> Result<Self> {
        Ok(Self {
            buffer_size_db,
            max_bitrate,
            avg_bitrate,
        })
    }
}

impl Atom for Btrt {
    const KIND: FourCC = FourCC::new(b"btrt");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let buffer_size_db = u32::decode(buf)?;
        let max_bitrate = u32::decode(buf)?;
        let avg_bitrate = u32::decode(buf)?;

        Ok(Btrt {
            buffer_size_db,
            max_bitrate,
            avg_bitrate,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.buffer_size_db.encode(buf)?;
        self.max_bitrate.encode(buf)?;
        self.avg_bitrate.encode(buf)?;
        Ok(())
    }
}
