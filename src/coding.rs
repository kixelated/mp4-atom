use std::ffi;

// Export these common types.
pub use bytes::Bytes;
pub use num::rational::Ratio;

use crate::{Buf, BufMut, Error, Result};

pub trait Decode: Sized {
    fn decode(buf: &mut Buf) -> Result<Self>;
}

pub trait DecodeTo {
    fn decode<T: Decode>(&mut self) -> Result<T>;
}

pub trait Encode {
    fn encode(&self, buf: &mut BufMut) -> Result<()>;
}

pub trait EncodeTo {
    fn encode<T: Encode>(&mut self, v: &T) -> Result<()>;
}

impl Decode for u8 {
    fn decode(buf: &mut Buf) -> Result<Self> {
        buf.u8()
    }
}

impl Decode for u16 {
    fn decode(buf: &mut Buf) -> Result<Self> {
        buf.u16()
    }
}

impl Decode for u32 {
    fn decode(buf: &mut Buf) -> Result<Self> {
        buf.u32()
    }
}

impl Decode for u64 {
    fn decode(buf: &mut Buf) -> Result<Self> {
        buf.u64()
    }
}

impl<const N: usize> Decode for [u8; N] {
    fn decode(buf: &mut Buf) -> Result<Self> {
        buf.fixed()
    }
}

impl<T: Decode> Decode for Vec<T> {
    fn decode(buf: &mut Buf) -> Result<Self> {
        let mut vec = Vec::new();
        while !buf.is_empty() {
            let item = T::decode(buf)?;
            vec.push(item);
        }

        Ok(vec)
    }
}

impl Decode for String {
    fn decode(buf: &mut Buf) -> Result<Self> {
        let mut bytes = Vec::new();
        loop {
            let byte = buf.u8()?;
            if byte == 0 {
                break;
            }

            bytes.push(byte);
        }

        let str = ffi::CString::new(bytes).map_err(|err| Error::InvalidString(err.to_string()))?;
        str.into_string()
            .map_err(|err| Error::InvalidString(err.to_string()))
    }
}

impl<T: Decode> Decode for Option<T> {
    fn decode(buf: &mut Buf) -> Result<Self> {
        if buf.is_empty() {
            Ok(Some(T::decode(buf)?))
        } else {
            Ok(None)
        }
    }
}

impl<T: Decode + Clone + num::Zero + num::Integer> Decode for Ratio<T> {
    fn decode(buf: &mut Buf) -> Result<Self> {
        let numer = T::decode(buf)?;
        let denom = T::decode(buf)?;
        if denom.is_zero() {
            return Err(Error::DivideByZero);
        }

        Ok(Ratio::new(numer, denom))
    }
}

impl Decode for Bytes {
    fn decode(buf: &mut Buf) -> Result<Self> {
        Ok(Bytes::copy_from_slice(buf.rest()))
    }
}

impl Encode for u8 {
    fn encode(&self, buf: &mut BufMut) -> Result<()> {
        buf.u8(*self);
        Ok(())
    }
}

impl Encode for u16 {
    fn encode(&self, buf: &mut BufMut) -> Result<()> {
        buf.u16(*self);
        Ok(())
    }
}

impl Encode for u32 {
    fn encode(&self, buf: &mut BufMut) -> Result<()> {
        buf.u32(*self);
        Ok(())
    }
}

impl Encode for u64 {
    fn encode(&self, buf: &mut BufMut) -> Result<()> {
        buf.u64(*self);
        Ok(())
    }
}

impl<const N: usize> Encode for [u8; N] {
    fn encode(&self, buf: &mut BufMut) -> Result<()> {
        buf.fixed(*self);
        Ok(())
    }
}

impl<T: Encode> Encode for &[T] {
    fn encode(&self, buf: &mut BufMut) -> Result<()> {
        for item in self.iter() {
            item.encode(buf)?;
        }

        Ok(())
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode(&self, buf: &mut BufMut) -> Result<()> {
        match self {
            Some(v) => v.encode(buf),
            None => Ok(()),
        }
    }
}

impl<T: Encode + num::Zero> Encode for Ratio<T> {
    fn encode(&self, buf: &mut BufMut) -> Result<()> {
        if self.denom().is_zero() {
            return Err(Error::DivideByZero);
        }

        self.numer().encode(buf)?;
        self.denom().encode(buf)
    }
}

impl Encode for &str {
    fn encode(&self, buf: &mut BufMut) -> Result<()> {
        buf.str(self);
        buf.u8(0);
        Ok(())
    }
}

impl<T: Encode> Encode for Vec<T> {
    fn encode(&self, buf: &mut BufMut) -> Result<()> {
        for item in self.iter() {
            item.encode(buf)?;
        }

        Ok(())
    }
}

impl Encode for Bytes {
    fn encode(&self, buf: &mut BufMut) -> Result<()> {
        buf.slice(self);
        Ok(())
    }
}

impl DecodeTo for Buf<'_> {
    fn decode<T: Decode>(&mut self) -> Result<T> {
        T::decode(self)
    }
}

impl EncodeTo for BufMut {
    fn encode<T: Encode>(&mut self, v: &T) -> Result<()> {
        v.encode(self)
    }
}
