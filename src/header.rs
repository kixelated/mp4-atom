use std::{fmt, io::Read};

use crate::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FourCC([u8; 4]);

impl FourCC {
    // Helper function to create a FourCC from a string literal
    // ex. FourCC::new(b"abcd")
    pub const fn new(value: &[u8; 4]) -> Self {
        FourCC(*value)
    }
}

impl From<u32> for FourCC {
    fn from(value: u32) -> Self {
        FourCC(value.to_be_bytes())
    }
}

impl From<FourCC> for u32 {
    fn from(cc: FourCC) -> Self {
        u32::from_be_bytes(cc.0)
    }
}

impl From<[u8; 4]> for FourCC {
    fn from(value: [u8; 4]) -> Self {
        FourCC(value)
    }
}

impl From<FourCC> for [u8; 4] {
    fn from(cc: FourCC) -> Self {
        cc.0
    }
}

impl From<&[u8; 4]> for FourCC {
    fn from(value: &[u8; 4]) -> Self {
        FourCC(*value)
    }
}

impl fmt::Display for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = String::from_utf8_lossy(&self.0);
        write!(f, "{}", s)
    }
}

impl fmt::Debug for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = String::from_utf8_lossy(&self.0);
        write!(f, "{}", s)
    }
}

impl Encode for FourCC {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        self.0.encode(buf)
    }
}

impl Decode for FourCC {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(FourCC(buf.decode()?))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Header {
    /// The name of the atom, always 4 bytes.
    pub kind: FourCC,

    // The size of the atom, **excluding** the header.
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
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
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
        Option::<Header>::read_from(r)?.ok_or(Error::UnexpectedEof)
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

        let size = u32::decode(&mut &buf[..4])?;
        let kind = FourCC::decode(&mut &buf[4..])?;

        let size = match size {
            0 => None,
            1 => {
                // Read another 8 bytes
                r.read_exact(&mut buf)?;
                let size = u64::decode(&mut &buf[..8])?;
                let size = size.checked_sub(16).ok_or(Error::InvalidSize)?;

                Some(size as usize)
            }
            _ => Some(size.checked_sub(8).ok_or(Error::InvalidSize)? as usize),
        };

        Ok(Some(Header { kind, size }))
    }
}
