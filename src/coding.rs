use std::ffi::CString;

// Export these traits
pub use bytes::{Buf, BufMut, Bytes};

// Export this common type.
pub use num::rational::Ratio;

use crate::{Error, Result};

pub trait Decode: Sized {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self>;
}
pub trait DecodeTo {
    fn decode<T: Decode>(&mut self) -> Result<T>;
}

pub trait Encode {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()>;
    fn encode_size(&self) -> usize;
}

pub trait EncodeTo {
    fn encode<T: Encode>(&mut self, v: &T) -> Result<()>;
}

impl Decode for u8 {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        if buf.remaining() < 1 {
            return Err(Error::LongRead);
        }

        Ok(buf.get_u8())
    }
}

impl Decode for u16 {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        if buf.remaining() < 2 {
            return Err(Error::LongRead);
        }

        Ok(buf.get_u16())
    }
}

impl Decode for u32 {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        if buf.remaining() < 4 {
            return Err(Error::LongRead);
        }

        Ok(buf.get_u32())
    }
}

impl Decode for u64 {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        if buf.remaining() < 8 {
            return Err(Error::LongRead);
        }

        Ok(buf.get_u64())
    }
}

impl<const N: usize> Decode for [u8; N] {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        if buf.remaining() < N {
            return Err(Error::LongRead);
        }

        let mut bytes = [0; N];
        buf.copy_to_slice(&mut bytes);

        Ok(bytes)
    }
}

impl<T: Decode> Decode for Vec<T> {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        let mut vec = Vec::new();
        while buf.has_remaining() {
            let item = T::decode(buf)?;
            vec.push(item);
        }

        Ok(vec)
    }
}

impl Decode for CString {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        let mut bytes = Vec::new();
        loop {
            let byte = buf.get_u8();
            if byte == 0 {
                break;
            }

            bytes.push(byte);
        }

        Ok(CString::new(bytes)?)
    }
}

impl<T: Decode> Decode for Option<T> {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        if buf.has_remaining() {
            Ok(Some(T::decode(buf)?))
        } else {
            Ok(None)
        }
    }
}

impl<T: Decode + Clone + num::Zero + num::Integer> Decode for Ratio<T> {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        let numer = T::decode(buf)?;
        let denom = T::decode(buf)?;
        if denom.is_zero() {
            return Err(Error::DivideByZero);
        }

        Ok(Ratio::new(numer, denom))
    }
}

impl Decode for Bytes {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(buf.copy_to_bytes(buf.remaining()))
    }
}

impl Encode for u8 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        if buf.remaining_mut() < 1 {
            return Err(Error::LongWrite);
        }

        buf.put_u8(*self);
        Ok(())
    }

    fn encode_size(&self) -> usize {
        1
    }
}

impl Encode for u16 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        if buf.remaining_mut() < 2 {
            return Err(Error::LongWrite);
        }

        buf.put_u16(*self);
        Ok(())
    }

    fn encode_size(&self) -> usize {
        2
    }
}

impl Encode for u32 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        if buf.remaining_mut() < 4 {
            return Err(Error::LongWrite);
        }

        buf.put_u32(*self);
        Ok(())
    }

    fn encode_size(&self) -> usize {
        4
    }
}

impl Encode for u64 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        if buf.remaining_mut() < 8 {
            return Err(Error::LongWrite);
        }

        buf.put_u64(*self);
        Ok(())
    }

    fn encode_size(&self) -> usize {
        8
    }
}

impl<const N: usize> Encode for [u8; N] {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        if buf.remaining_mut() < N {
            return Err(Error::LongWrite);
        }

        buf.put_slice(self);
        Ok(())
    }

    fn encode_size(&self) -> usize {
        N
    }
}

impl<T: Encode> Encode for &[T] {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        for item in self.iter() {
            item.encode(buf)?;
        }

        Ok(())
    }

    fn encode_size(&self) -> usize {
        self.into_iter().map(Encode::encode_size).sum()
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        match self {
            Some(v) => v.encode(buf),
            None => Ok(()),
        }
    }

    fn encode_size(&self) -> usize {
        match self {
            Some(v) => v.encode_size(),
            None => 0,
        }
    }
}

impl<T: Encode + num::Zero> Encode for Ratio<T> {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        if self.denom().is_zero() {
            return Err(Error::DivideByZero);
        }

        self.numer().encode(buf)?;
        self.denom().encode(buf)
    }

    fn encode_size(&self) -> usize {
        self.numer().encode_size() + self.denom().encode_size()
    }
}

impl Encode for &str {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        // Not using Encode for &[u8] because this is faster
        let bytes = self.as_bytes();
        if buf.remaining_mut() < bytes.len() {
            return Err(Error::LongWrite);
        }

        buf.put_slice(bytes);
        Ok(())
    }

    fn encode_size(&self) -> usize {
        self.as_bytes().len()
    }
}

impl<T: Encode> Encode for Vec<T> {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        for item in self.iter() {
            item.encode(buf)?;
        }

        Ok(())
    }

    fn encode_size(&self) -> usize {
        self.iter().map(Encode::encode_size).sum()
    }
}

impl Encode for Bytes {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        if buf.remaining_mut() < self.len() {
            return Err(Error::LongWrite);
        }

        buf.put_slice(self);
        Ok(())
    }

    fn encode_size(&self) -> usize {
        self.len()
    }
}

impl<B: Buf> DecodeTo for &mut B {
    fn decode<T: Decode>(&mut self) -> Result<T> {
        T::decode(self)
    }
}

impl<B: BufMut> EncodeTo for &mut B {
    fn encode<T: Encode>(&mut self, v: &T) -> Result<()> {
        v.encode(self)
    }
}
