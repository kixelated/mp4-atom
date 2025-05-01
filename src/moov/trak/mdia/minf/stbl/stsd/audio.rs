use crate::coding::{Decode, Encode};
use crate::{Buf, BufMut, Error, FixedPoint, Result};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Audio {
    pub data_reference_index: u16,
    pub channel_count: u16,
    pub sample_size: u16,
    pub sample_rate: FixedPoint<u16>,
}

impl Encode for Audio {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        0u32.encode(buf)?; // reserved
        0u16.encode(buf)?; // reserved
        self.data_reference_index.encode(buf)?;
        0u16.encode(buf)?; // version
        0u16.encode(buf)?; // reserved
        0u32.encode(buf)?; // reserved
        self.channel_count.encode(buf)?;
        self.sample_size.encode(buf)?;
        0u32.encode(buf)?; // reserved
        self.sample_rate.encode(buf)?;

        Ok(())
    }
}
impl Decode for Audio {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        u32::decode(buf)?; // reserved
        u16::decode(buf)?; // reserved
        let data_reference_index = u16::decode(buf)?;
        let version = u16::decode(buf)?;
        u16::decode(buf)?; // reserved
        u32::decode(buf)?; // reserved
        let channel_count = u16::decode(buf)?;
        let sample_size = u16::decode(buf)?;
        u32::decode(buf)?; // pre-defined, reserved
        let sample_rate = FixedPoint::decode(buf)?;

        match version {
            0 => {}
            1 => {
                // Quicktime sound sample description version 1.
                u64::decode(buf)?;
                u64::decode(buf)?;
            }
            2 => {
                // Quicktime sound sample description version 2.
                u32::decode(buf)?;
                let _sample_rate = u64::decode(buf)?;
                let _channel_count = u32::decode(buf)?;
                <[u8; 20]>::decode(buf)?;
            }
            n => return Err(Error::UnknownQuicktimeVersion(n)),
        }

        Ok(Self {
            data_reference_index,
            channel_count,
            sample_size,
            sample_rate,
        })
    }
}
