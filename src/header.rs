use std::{fmt, io::Read};

use crate::*;


/// A atom header, which contains the atom's kind and size.
#[derive(Debug, Clone, Copy)]
pub struct Header {
    /// The name of the atom, always 4 bytes.
    pub kind: FourCC,

    /// The size of the atom, **excluding** the header.
    /// This is optional when the atom extends to the end of the file.
    pub size: Option<usize>,
}

impl Encode for Header {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        match self.size.map(|size| size + 8) {
            Some(size) if size > u32::MAX as usize => {
                1u32.encode(buf)?;
                self.kind.encode(buf)?;

                // Have to include the size of this extra field
                ((size + 8) as u64).encode(buf)
            }
            Some(size) => {
                (size as u32).encode(buf)?;
                self.kind.encode(buf)
            }
            None => {
                0u32.encode(buf)?;
                self.kind.encode(buf)
            }
        }
    }
}

impl Decode for Header {
    fn decode(buf: &mut Bytes) -> Result<Self> {
        let size = u32::decode(buf)?;
        let kind = FourCC::decode(buf)?;

        Ok(match size {
            0 => Self { kind, size: None },
            1 => {
                let size = u64::decode(buf)?;
                let size = size.checked_sub(16).ok_or(Error::InvalidSize)?;

                Self {
                    kind,
                    size: Some(size as usize),
                }
            }
            _ => {
                let size = size.checked_sub(8).ok_or(Error::InvalidSize)?;
                Self {
                    kind,
                    size: Some(size as usize),
                }
            }
        })
    }
}

impl ReadFrom for Header {
    fn read_from<R: Read>(r: &mut R) -> Result<Self> {
        <Option<Header> as ReadFrom>::read_from(r)?.ok_or(Error::UnexpectedEof)
    }
}

impl ReadFrom for Option<Header> {
    fn read_from<R: Read>(r: &mut R) -> Result<Self> {
        let mut buf = [0u8; 8];
        let n = r.read(&mut buf)?;
        if n == 0 {
            return Ok(None);
        }

        r.read_exact(&mut buf[n..])?;

        let size = u32::from_be_bytes(buf[0..4].try_into().unwrap());
        let kind = u32::from_be_bytes(buf[4..8].try_into().unwrap()).into();

        let size = match size {
            0 => None,
            1 => {
                // Read another 8 bytes
                r.read_exact(&mut buf)?;
                let size = u64::from_be_bytes(buf);
                let size = size.checked_sub(16).ok_or(Error::InvalidSize)?;

                Some(size as usize)
            }
            _ => Some(size.checked_sub(8).ok_or(Error::InvalidSize)? as usize),
        };

        Ok(Some(Header { kind, size }))
    }
}

#[cfg(feature = "tokio")]
impl AsyncReadFrom for Header {
    async fn read_from<R: tokio::io::AsyncRead + Unpin>(r: &mut R) -> Result<Self> {
        <Option<Header> as AsyncReadFrom>::read_from(r)
            .await?
            .ok_or(Error::UnexpectedEof)
    }
}

#[cfg(feature = "tokio")]
impl AsyncReadFrom for Option<Header> {
    async fn read_from<R: tokio::io::AsyncRead + Unpin>(r: &mut R) -> Result<Self> {
        use tokio::io::AsyncReadExt;

        let mut buf = [0u8; 8];
        let n = r.read(&mut buf).await?;
        if n == 0 {
            return Ok(None);
        }

        r.read_exact(&mut buf[n..]).await?;

        let size = u32::from_be_bytes(buf[0..4].try_into().unwrap());
        let kind = u32::from_be_bytes(buf[4..8].try_into().unwrap()).into();

        let size = match size {
            0 => None,
            1 => {
                // Read another 8 bytes
                r.read_exact(&mut buf).await?;
                let size = u64::from_be_bytes(buf);
                let size = size.checked_sub(16).ok_or(Error::InvalidSize)?;

                Some(size as usize)
            }
            _ => Some(size.checked_sub(8).ok_or(Error::InvalidSize)? as usize),
        };

        Ok(Some(Header { kind, size }))
    }
}

impl Header {
    pub fn decode_any(&self, buf: &mut Bytes) -> Result<Any> {
        Any::decode_atom(self, buf)
    }

    pub fn decode_atom<T: Atom>(&self, buf: &mut Bytes) -> Result<T> {
        let size = self.size.unwrap_or(buf.remaining());
        let buf = &mut buf.decode_exact(size)?;

        let atom = match T::decode_atom(buf) {
            Ok(atom) => atom,
            Err(Error::OutOfBounds) => return Err(Error::OverDecode(T::KIND)),
            Err(Error::ShortRead) => return Err(Error::UnderDecode(T::KIND)),
            Err(err) => return Err(err),
        };

        if buf.has_remaining() {
            return Err(Error::UnderDecode(T::KIND));
        }

        Ok(atom)
    }
}
