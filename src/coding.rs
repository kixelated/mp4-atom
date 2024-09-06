use std::ffi;

use crate::*;

impl Decode for u8 {
    fn decode(buf: &mut Bytes) -> Result<Self> {
        if !buf.has_remaining() {
            return Err(Error::OutOfBounds);
        }

        Ok(buf.get_u8())
    }
}

impl Decode for i8 {
    fn decode(buf: &mut Bytes) -> Result<Self> {
        if !buf.has_remaining() {
            return Err(Error::OutOfBounds);
        }

        Ok(buf.get_i8())
    }
}

impl Decode for u16 {
    fn decode(buf: &mut Bytes) -> Result<Self> {
        Ok(Self::from_be_bytes(buf.decode()?))
    }
}

impl Decode for i16 {
    fn decode(buf: &mut Bytes) -> Result<Self> {
        Ok(Self::from_be_bytes(buf.decode()?))
    }
}

impl Decode for u32 {
    fn decode(buf: &mut Bytes) -> Result<Self> {
        Ok(Self::from_be_bytes(buf.decode()?))
    }
}

impl Decode for i32 {
    fn decode(buf: &mut Bytes) -> Result<Self> {
        Ok(Self::from_be_bytes(buf.decode()?))
    }
}

impl Decode for u64 {
    fn decode(buf: &mut Bytes) -> Result<Self> {
        Ok(Self::from_be_bytes(buf.decode()?))
    }
}

impl Decode for i64 {
    fn decode(buf: &mut Bytes) -> Result<Self> {
        Ok(Self::from_be_bytes(buf.decode()?))
    }
}

impl<const N: usize> Decode for [u8; N] {
    fn decode(buf: &mut Bytes) -> Result<Self> {
        if buf.remaining() < N {
            return Err(Error::OutOfBounds);
        }

        let mut bytes = [0; N];
        buf.copy_to_slice(&mut bytes);
        Ok(bytes)
    }
}

impl<T: Decode> Decode for Vec<T> {
    fn decode(buf: &mut Bytes) -> Result<Self> {
        let mut vec = Vec::new();
        while buf.has_remaining() {
            let item = buf.decode()?;
            vec.push(item);
        }

        Ok(vec)
    }
}

impl Decode for String {
    fn decode(buf: &mut Bytes) -> Result<Self> {
        let mut bytes = Vec::new();
        loop {
            let byte = buf.decode()?;
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
    fn decode(buf: &mut Bytes) -> Result<Self> {
        if buf.has_remaining() {
            Ok(Some(buf.decode()?))
        } else {
            Ok(None)
        }
    }
}

impl Decode for Bytes {
    fn decode(buf: &mut Bytes) -> Result<Self> {
        Ok(buf.copy_to_bytes(buf.remaining()))
    }
}

impl Encode for u8 {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        buf.put_u8(*self);
        Ok(())
    }
}

impl Encode for i8 {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        buf.put_i8(*self);
        Ok(())
    }
}

impl Encode for i16 {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        buf.put_i16(*self);
        Ok(())
    }
}

impl Encode for u16 {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        buf.put_u16(*self);
        Ok(())
    }
}

impl Encode for u32 {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        buf.put_u32(*self);
        Ok(())
    }
}

impl Encode for i32 {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        buf.put_i32(*self);
        Ok(())
    }
}

impl Encode for u64 {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        buf.put_u64(*self);
        Ok(())
    }
}

impl Encode for i64 {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        buf.put_i64(*self);
        Ok(())
    }
}

impl<const N: usize> Encode for [u8; N] {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        buf.put_slice(self);
        Ok(())
    }
}

impl<T: Encode> Encode for &[T] {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        for item in self.iter() {
            item.encode(buf)?;
        }

        Ok(())
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        match self {
            Some(v) => v.encode(buf),
            None => Ok(()),
        }
    }
}

impl Encode for &str {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        buf.put_slice(self.as_bytes());
        buf.put_u8(0);
        Ok(())
    }
}

impl<T: Encode> Encode for Vec<T> {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        for item in self.iter() {
            item.encode(buf)?;
        }

        Ok(())
    }
}

impl Encode for Bytes {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        buf.put_slice(self);
        Ok(())
    }
}

impl DecodeFrom for Bytes {
    fn decode<T: Decode>(&mut self) -> Result<T> {
        T::decode(self)
    }

    fn decode_exact<T: Decode>(&mut self, size: usize) -> Result<T> {
        T::decode_exact(self, size)
    }
}

impl EncodeTo for BytesMut {
    fn encode<T: Encode>(&mut self, v: &T) -> Result<()> {
        v.encode(self)
    }
}
