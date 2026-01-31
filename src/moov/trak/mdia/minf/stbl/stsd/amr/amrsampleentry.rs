use crate::coding::{Decode, Encode};
use crate::{Buf, BufMut, Result};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmrSampleEntry {
    pub data_reference_index: u16,
    pub timescale: u16,
}

impl Encode for AmrSampleEntry {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        [0u8; 6].encode(buf)?; // Reserved_6
        self.data_reference_index.encode(buf)?;
        [0u8; 8].encode(buf)?; // Reserved_8
        2u16.encode(buf)?; // Reserved_2
        16u16.encode(buf)?; // Reserved_2
        [0u8; 4].encode(buf)?; // Reserved_4
        self.timescale.encode(buf)?;
        0u16.encode(buf)?; // Reserved_2
        Ok(())
    }
}
impl Decode for AmrSampleEntry {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        u32::decode(buf)?; // first part of reserved_6
        u16::decode(buf)?; // second part of reserved_6
        let data_reference_index = u16::decode(buf)?;
        u32::decode(buf)?; // first half of reserved_8
        u32::decode(buf)?; // second half of reserved_8
        u16::decode(buf)?; // reserved_2
        u16::decode(buf)?; // reserved_2
        u32::decode(buf)?; // reserved_4
        let timescale = u16::decode(buf)?;
        u16::decode(buf)?; // reserved_2

        Ok(Self {
            data_reference_index,
            timescale,
        })
    }
}
