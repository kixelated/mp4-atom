use std::fmt;

pub use crate::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct u24([u8; 3]);

impl Decode for u24 {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
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
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
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

// The top N bits are the integer part, the bottom N bits are the fractional part.
// Somebody who cares should implement some math stuff.
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct FixedPoint<T> {
    int: T,
    dec: T,
}

impl<T: Copy> FixedPoint<T> {
    pub fn new(int: T, dec: T) -> Self {
        Self { int, dec }
    }

    pub fn integer(&self) -> T {
        self.int
    }

    pub fn decimal(&self) -> T {
        self.dec
    }
}

impl<T: Decode> Decode for FixedPoint<T> {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Self {
            int: T::decode(buf)?,
            dec: T::decode(buf)?,
        })
    }
}

impl<T: Encode> Encode for FixedPoint<T> {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        buf.encode(&self.int)?;
        buf.encode(&self.dec)
    }
}

impl<T: num::Zero> From<T> for FixedPoint<T> {
    fn from(value: T) -> Self {
        Self {
            int: value,
            dec: T::zero(),
        }
    }
}

impl<T> fmt::Debug for FixedPoint<T>
where
    T: num::Zero + fmt::Debug + PartialEq + Copy,
    f64: From<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.dec.is_zero() {
            write!(f, "{:?}", self.int)
        } else {
            write!(f, "{:?}", f64::from(self.int) / f64::from(self.dec))
        }
    }
}

// 32 bytes max
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Compressor(String);

impl From<&str> for Compressor {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for Compressor {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<Compressor> for String {
    fn from(value: Compressor) -> Self {
        value.0
    }
}

impl AsRef<str> for Compressor {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Encode for Compressor {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        let name = self.0.as_bytes();
        let max = name.len().min(31);
        buf.put_slice(&name[..max]);
        for _ in max..31 {
            buf.put_u8(0);
        }

        Ok(())
    }
}

impl Decode for Compressor {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        let mut name = [0; 32];
        buf.copy_to_slice(&mut name);

        let name = String::from_utf8_lossy(&name)
            .trim_end_matches('\0')
            .to_string();

        Ok(Self(name))
    }
}
