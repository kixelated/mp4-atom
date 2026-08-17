use crate::coding::{Decode, Encode};
use crate::{Buf, BufMut, Result};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
/// MetaDataSampleEntry base fields.
///
/// Used for meta data. Its essentially just the standard sample entry body.
///
/// See ISO/IEC 14496-12:2026 Section 12.3.3.2 and 8.5.2.2.
pub struct MetaData {
    pub data_reference_index: u16,
}

impl Encode for MetaData {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        0u32.encode(buf)?; // reserved
        0u16.encode(buf)?; // reserved
        self.data_reference_index.encode(buf)?;
        Ok(())
    }
}
impl Decode for MetaData {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        u32::decode(buf)?; // reserved
        u16::decode(buf)?; // reserved
        let data_reference_index = u16::decode(buf)?;

        Ok(Self {
            data_reference_index,
        })
    }
}
