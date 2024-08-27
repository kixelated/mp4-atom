pub use crate::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct u24([u8; 3]);

impl Decode for u24 {
    fn decode(buf: &mut Bytes) -> Result<Self> {
        Ok(Self(buf.decode()?))
    }
}

impl Encode for u24 {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        buf.encode(&self.0)
    }
}

impl From<u24> for u32 {
    fn from(value: u24) -> Self {
        u32::from_be_bytes([0, value.0[0], value.0[1], value.0[2]])
    }
}

impl TryFrom<u32> for u24 {
    type Error = std::array::TryFromSliceError;

    fn try_from(value: u32) -> std::result::Result<Self, Self::Error> {
        Ok(Self(value.to_be_bytes()[1..].try_into()?))
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct u48([u8; 6]);

impl Decode for u48 {
    fn decode(buf: &mut Bytes) -> Result<Self> {
        Ok(Self(buf.decode()?))
    }
}

impl Encode for u48 {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        buf.encode(&self.0)
    }
}

impl TryFrom<u64> for u48 {
    type Error = std::array::TryFromSliceError;

    fn try_from(value: u64) -> std::result::Result<Self, Self::Error> {
        Ok(Self(value.to_be_bytes()[2..].try_into()?))
    }
}

impl From<u48> for u64 {
    fn from(value: u48) -> Self {
        u64::from_be_bytes([
            0, 0, value.0[0], value.0[1], value.0[2], value.0[3], value.0[4], value.0[5],
        ])
    }
}
