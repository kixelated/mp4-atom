use std::ffi;
use std::ops::Range;

use crate::*;

/// Decode a type from a buffer.
pub trait Decode: Sized {
    /// Decode the type from the buffer.
    fn decode<B: Buf>(buf: &mut B) -> Result<Self>;

    /// Helper: Decode exactly size bytes from the buffer.
    fn decode_exact<B: Buf>(buf: &mut B, size: usize) -> Result<Self> {
        if buf.remaining() < size {
            return Err(Error::OutOfBounds);
        }

        let mut inner = buf.slice(size);
        let res = Self::decode(&mut inner)?;

        if inner.has_remaining() {
            return Err(Error::ShortRead);
        }

        buf.advance(size);

        Ok(res)
    }
}

/// Decode a type from a buffer if we have enough data.
pub trait DecodeMaybe: Sized {
    fn decode_maybe<B: Buf>(buf: &mut B) -> Result<Option<Self>>;
}

/// Decode an atom using the provided header
pub trait DecodeAtom: Sized {
    fn decode_atom<B: Buf>(header: &Header, buf: &mut B) -> Result<Self>;
}

/// Encode a type to a buffer.
//
// Why not BufMut?
// Well it's because we need to write the size of each atom.
// If we use BufMut, we can't seek backwards so we have to calculate it upfront.
// If we use BufMut or Vec, then we can write 0 for the size, then write the atom, then go back and fix the size.
pub trait Encode {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()>;

    #[cfg(test)]
    fn assert_encode_decode(&self)
    where
        Self: std::fmt::Debug + PartialEq + Decode,
    {
        let mut buf = Vec::new();
        Self::encode(self, &mut buf).unwrap();

        let mut cursor = std::io::Cursor::new(&buf);
        let decoded = Self::decode(&mut cursor).unwrap();

        assert_eq!(self, &decoded, "different decoded result");
    }
}

impl Decode for u8 {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Self::from_be_bytes(<[u8; 1]>::decode(buf)?))
    }
}

impl Decode for i8 {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Self::from_be_bytes(<[u8; 1]>::decode(buf)?))
    }
}

impl Decode for u16 {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Self::from_be_bytes(<[u8; 2]>::decode(buf)?))
    }
}

impl Decode for i16 {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Self::from_be_bytes(<[u8; 2]>::decode(buf)?))
    }
}

impl Decode for u32 {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Self::from_be_bytes(<[u8; 4]>::decode(buf)?))
    }
}

impl Decode for i32 {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Self::from_be_bytes(<[u8; 4]>::decode(buf)?))
    }
}

impl Decode for u64 {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Self::from_be_bytes(<[u8; 8]>::decode(buf)?))
    }
}

impl Decode for i64 {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Self::from_be_bytes(<[u8; 8]>::decode(buf)?))
    }
}

impl<const N: usize> Decode for [u8; N] {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        if buf.remaining() < N {
            return Err(Error::OutOfBounds);
        }

        let mut v = [0u8; N];
        v.copy_from_slice(buf.slice(N));
        buf.advance(N);

        Ok(v)
    }
}

impl<T: Decode> Decode for Range<T> {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        let start = T::decode(buf)?;
        let end = T::decode(buf)?;
        Ok(start..end)
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

impl Decode for String {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        let mut bytes = Vec::new();
        while buf.has_remaining() {
            let byte = u8::decode(buf)?;
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

impl Encode for u8 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.to_be_bytes().encode(buf)
    }
}

impl Encode for i8 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.to_be_bytes().encode(buf)
    }
}

impl Encode for i16 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.to_be_bytes().encode(buf)
    }
}

impl Encode for u16 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.to_be_bytes().encode(buf)
    }
}

impl Encode for u32 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.to_be_bytes().encode(buf)
    }
}

impl Encode for i32 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.to_be_bytes().encode(buf)
    }
}

impl Encode for u64 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.to_be_bytes().encode(buf)
    }
}

impl Encode for i64 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.to_be_bytes().encode(buf)
    }
}

impl<const N: usize> Encode for [u8; N] {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        buf.append_slice(self);
        Ok(())
    }
}

impl<T: Encode> Encode for &[T] {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        for item in self.iter() {
            item.encode(buf)?;
        }

        Ok(())
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        match self {
            Some(v) => v.encode(buf),
            None => Ok(()),
        }
    }
}

impl Encode for &str {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.as_bytes().encode(buf)?;
        0u8.encode(buf)?;
        Ok(())
    }
}

impl<T: Encode> Encode for Range<T> {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.start.encode(buf)?;
        self.end.encode(buf)
    }
}

impl<T: Encode> Encode for Vec<T> {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        for item in self.iter() {
            item.encode(buf)?;
        }

        Ok(())
    }
}

/// Decode a UTF-8 string preceded by its length in bytes, stored as `T`.
pub(crate) fn decode_length_prefixed_utf8_string<T: Decode + Into<usize>>(
    buf: &mut impl Buf,
) -> Result<String> {
    let len = T::decode(buf)?;
    decode_utf8_string_with_length(buf, len.into())
}

/// Decode a UTF-8 string filling the rest of the buffer.
pub(crate) fn decode_utf8_string(buf: &mut impl Buf) -> Result<String> {
    decode_utf8_string_with_length(buf, buf.remaining())
}

/// Decode a UTF-8 string of `len` bytes.
pub(crate) fn decode_utf8_string_with_length(buf: &mut impl Buf, len: usize) -> Result<String> {
    if len > buf.remaining() {
        return Err(Error::OutOfBounds);
    }

    let text = String::from_utf8(buf.slice(len).to_vec())
        .map_err(|err| Error::InvalidString(err.to_string()))?;

    buf.advance(len);
    Ok(text)
}

/// Encode a UTF-8 string preceded by its length in bytes, stored as `T`. `kind`
/// is only used to report a string whose length doesn't fit in `T`.
pub(crate) fn encode_length_prefixed_string<T: Encode + TryFrom<usize>>(
    buf: &mut impl BufMut,
    text: &str,
    kind: FourCC,
) -> Result<()> {
    let len: T = text.len().try_into().map_err(|_| Error::TooLarge(kind))?;
    len.encode(buf)?;
    buf.append_slice(text.as_bytes());
    Ok(())
}

/// Decode a string preceded by its length in bytes, stored as `T`, see
/// [`decode_bom_string_with_length`].
pub(crate) fn decode_length_prefixed_bom_string<T: Decode + Into<usize>>(
    buf: &mut impl Buf,
) -> Result<String> {
    let len = T::decode(buf)?;
    decode_bom_string_with_length(buf, len.into())
}

/// Decode `len` bytes of text that is UTF-8, or UTF-16 (either byte order) if
/// it starts with a byte order mark, as used by 3GPP timed text.
pub(crate) fn decode_bom_string_with_length(buf: &mut impl Buf, len: usize) -> Result<String> {
    if len > buf.remaining() {
        return Err(Error::OutOfBounds);
    }

    let bytes = buf.slice(len);
    let text = match bytes {
        [0xFE, 0xFF, rest @ ..] => decode_utf16_units(rest, u16::from_be_bytes),
        [0xFF, 0xFE, rest @ ..] => decode_utf16_units(rest, u16::from_le_bytes),
        _ => String::from_utf8(bytes.to_vec()).map_err(|err| err.to_string()),
    }
    .map_err(Error::InvalidString)?;

    buf.advance(len);
    Ok(text)
}

fn decode_utf16_units(
    bytes: &[u8],
    from_bytes: fn([u8; 2]) -> u16,
) -> std::result::Result<String, String> {
    let (pairs, rest) = bytes.as_chunks::<2>();

    if !rest.is_empty() {
        return Err("UTF-16 payload has an odd byte length".to_string());
    }

    let units: Vec<u16> = pairs.iter().cloned().map(from_bytes).collect();
    String::from_utf16(&units).map_err(|err| err.to_string())
}

#[cfg(test)]
mod string_helper_tests {
    use super::*;

    #[test]
    fn test_length_prefixed_string_roundtrip() {
        let mut buf = Vec::new();
        encode_length_prefixed_string::<u8>(&mut buf, "héllo", FourCC::new(b"test")).unwrap();
        assert_eq!(buf[0] as usize, "héllo".len());

        let mut slice = buf.as_slice();
        assert_eq!(
            decode_length_prefixed_utf8_string::<u8>(&mut slice).unwrap(),
            "héllo"
        );
        assert!(slice.is_empty());
    }

    #[test]
    fn test_length_prefixed_string_errors() {
        let too_long = "a".repeat(256);
        let err =
            encode_length_prefixed_string::<u8>(&mut Vec::new(), &too_long, FourCC::new(b"test"));
        assert!(matches!(err, Err(Error::TooLarge(_))));

        // Declared length exceeds the remaining bytes.
        let err = decode_length_prefixed_utf8_string::<u8>(&mut [5u8, b'a'].as_slice());
        assert!(matches!(err, Err(Error::OutOfBounds)));
    }

    #[test]
    fn test_bom_string() {
        let mut utf8 = b"Hi".as_slice();
        assert_eq!(decode_bom_string_with_length(&mut utf8, 2).unwrap(), "Hi");
        assert!(utf8.is_empty());

        let be = [0xFE, 0xFF, 0, b'H', 0, b'i'];
        assert_eq!(
            decode_bom_string_with_length(&mut be.as_slice(), 6).unwrap(),
            "Hi"
        );

        let le = [0xFF, 0xFE, b'H', 0, b'i', 0];
        assert_eq!(
            decode_bom_string_with_length(&mut le.as_slice(), 6).unwrap(),
            "Hi"
        );
    }

    #[test]
    fn test_bom_string_errors() {
        // Longer than the buffer.
        let err = decode_bom_string_with_length(&mut b"Hi".as_slice(), 3);
        assert!(matches!(err, Err(Error::OutOfBounds)));

        // Odd UTF-16 payload, and the buffer is left untouched on failure.
        let odd = [0xFE, 0xFF, 0, b'H', 0];
        let mut slice = odd.as_slice();
        assert!(matches!(
            decode_bom_string_with_length(&mut slice, 5),
            Err(Error::InvalidString(_))
        ));
        assert_eq!(slice.len(), 5);
    }
}
