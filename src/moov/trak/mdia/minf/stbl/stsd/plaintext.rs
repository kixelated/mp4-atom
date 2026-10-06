use crate::coding::{Decode, Encode};
use crate::{Buf, BufMut, Result};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
/// Plain text sample entry.
///
/// Used for timed text. Its essentially just the standard sample entry body.
///
/// See ISO/IEC 14496-12:2022 Section 12.5.3.
pub struct PlainText {
    pub data_reference_index: u16,
}

impl Encode for PlainText {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        0u32.encode(buf)?; // reserved
        0u16.encode(buf)?; // reserved
        self.data_reference_index.encode(buf)?;
        Ok(())
    }
}
impl Decode for PlainText {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        u32::decode(buf)?; // reserved
        u16::decode(buf)?; // reserved
        let data_reference_index = u16::decode(buf)?;

        Ok(Self {
            data_reference_index,
        })
    }
}
