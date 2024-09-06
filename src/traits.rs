use std::io::{Read, Write};

use crate::Header;

use super::{Error, Result};

// Re-export these common types.
pub use bytes::{Buf, BufMut, Bytes, BytesMut};

/// Decode a type from a buffer.
//
// Why not Buf?
// I did try it, but it prevents the DecodeFrom trait because of a weird recursion bug.
// It's easier in general to just use Bytes to avoid traits anyway, even if it's less flexible.
pub trait Decode: Sized {
    fn decode(buf: &mut Bytes) -> Result<Self>;

    fn decode_exact(buf: &mut Bytes, size: usize) -> Result<Self> {
        if buf.remaining() < size {
            return Err(Error::OutOfBounds);
        }

        let buf = &mut buf.copy_to_bytes(size);
        let res = Self::decode(buf)?;

        if buf.remaining() > 0 {
            return Err(Error::ShortRead);
        }

        Ok(res)
    }
}

/// A helper that lets you call `buf.decode()` for any type that implements Decode.
/// This will automatically infer T from the context, reducing boilerplate significantly.
pub trait DecodeFrom {
    fn decode<T: Decode>(&mut self) -> Result<T>;
    fn decode_exact<T: Decode>(&mut self, size: usize) -> Result<T>;
}

/// Decode an atom using the provided header
pub trait DecodeAtom: Sized {
    fn decode_atom(header: &Header, buf: &mut Bytes) -> Result<Self>;
}

/// Encode a type to a buffer.
//
// Why not BufMut?
// Well it's because we need to write the size of each atom.
// If we use BufMut, we can't seek backwards so we have to calculate it upfront.
// If we use BytesMut or Vec, then we can write 0 for the size, then write the atom, then go back and fix the size.
pub trait Encode {
    fn encode(&self, buf: &mut BytesMut) -> Result<()>;
}

/// A helper that lets you call `buf.encode(&value)` for any type that implements Encode.
// Not as useful but still nice to have.
pub trait EncodeTo {
    fn encode<T: Encode>(&mut self, v: &T) -> Result<()>;
}

/// Read a type from a reader.
pub trait ReadFrom: Sized {
    fn read_from<R: Read>(r: &mut R) -> Result<Self>;
}

/// Read an atom from a reader provided the header.
pub trait ReadAtom: Sized {
    fn read_atom<R: Read>(header: &Header, r: &mut R) -> Result<Self>;
}

/// Write a type to a writer.
pub trait WriteTo {
    fn write_to<W: Write>(&self, w: &mut W) -> Result<()>;
}

#[cfg(feature = "tokio")]
pub trait AsyncReadFrom: Sized {
    #[allow(async_fn_in_trait)]
    async fn read_from<R: tokio::io::AsyncRead + Unpin>(r: &mut R) -> Result<Self>;
}

#[cfg(feature = "tokio")]
pub trait AsyncWriteTo {
    #[allow(async_fn_in_trait)]
    async fn write_to<W: tokio::io::AsyncWrite + Unpin>(&self, w: &mut W) -> Result<()>;
}

#[cfg(feature = "tokio")]
pub trait AsyncReadAtom: Sized {
    #[allow(async_fn_in_trait)]
    async fn read_atom<R: tokio::io::AsyncRead + Unpin>(header: &Header, r: &mut R)
        -> Result<Self>;
}

impl<T: Encode> WriteTo for T {
    fn write_to<W: Write>(&self, w: &mut W) -> Result<()> {
        // TODO We should avoid allocating a buffer here.
        let mut buf = BytesMut::new();
        self.encode(&mut buf)?;
        Ok(w.write_all(&buf)?)
    }
}

#[cfg(feature = "tokio")]
impl<T: Encode> AsyncWriteTo for T {
    async fn write_to<W: tokio::io::AsyncWrite + Unpin>(&self, w: &mut W) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        // TODO We should avoid allocating a buffer here.
        let mut buf = BytesMut::new();
        self.encode(&mut buf)?;
        Ok(w.write_all(&buf).await?)
    }
}
