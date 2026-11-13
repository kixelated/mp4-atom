use std::io::{Cursor, Read};

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
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let too_large = || Error::TooLarge(self.kind);

        match self.size {
            Some(size) => {
                if size > u32::MAX as usize - 8 {
                    1u32.encode(buf)?;
                    self.kind.encode(buf)?;

                    // Have to include the size of this extra field
                    let size = size.checked_add(16).ok_or_else(too_large)?;
                    u64::try_from(size).map_err(|_| too_large())?.encode(buf)
                } else {
                    (size as u32 + 8).encode(buf)?;
                    self.kind.encode(buf)
                }
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

        let size = match size {
            0 => None,
            // The real size follows in another 8 bytes.
            1 => Some(body_size(kind, u64::decode(buf)?, 16)?),
            _ => Some(body_size(kind, size.into(), 8)?),
        };

        Ok(Self { kind, size })
    }
}

impl DecodeMaybe for Header {
    fn decode_maybe<B: Buf>(buf: &mut B) -> Result<Option<Self>> {
        if buf.remaining() < 8 {
            return Ok(None);
        }

        let size = u32::from_be_bytes(buf.slice(4).try_into().unwrap());
        if size == 1 && buf.remaining() < 16 {
            return Ok(None);
        }

        Ok(Some(Self::decode(buf)?))
    }
}

impl ReadFrom for Header {
    fn read_from<R: Read + ?Sized>(r: &mut R) -> Result<Self> {
        <Option<Header> as ReadFrom>::read_from(r)?.ok_or(Error::UnexpectedEof)
    }
}

impl ReadFrom for Option<Header> {
    fn read_from<R: Read + ?Sized>(r: &mut R) -> Result<Self> {
        let mut buf = [0u8; 16];
        let n = r.read(&mut buf[..8])?;
        if n == 0 {
            return Ok(None);
        }

        r.read_exact(&mut buf[n..8])?;

        // A size of 1 means the real size follows in another 8 bytes.
        let len = header_len(&buf);
        r.read_exact(&mut buf[8..len])?;

        Header::decode(&mut &buf[..len]).map(Some)
    }
}

/// The length of a header, given its first 8 bytes.
pub(crate) fn header_len(buf: &[u8]) -> usize {
    match buf[0..4] {
        [0, 0, 0, 1] => 16,
        _ => 8,
    }
}

/// Convert the size of an atom, as stored in its header, into the size of its body.
fn body_size(kind: FourCC, size: u64, header_len: u64) -> Result<usize> {
    let size = size.checked_sub(header_len).ok_or(Error::InvalidSize)?;
    usize::try_from(size).map_err(|_| Error::TooLarge(kind))
}

// Utility methods
impl Header {
    pub(crate) fn read_body<R: Read + ?Sized>(&self, r: &mut R) -> Result<Cursor<Vec<u8>>> {
        // TODO This allocates on the heap.
        // Ideally, we should use ReadFrom instead of Decode to avoid this.

        // Don't use `with_capacity` on an untrusted size
        // We allocate at most 4096 bytes upfront and grow as needed
        let cap = self.size.unwrap_or(0).min(4096);
        let mut buf = Vec::with_capacity(cap);

        match self.size {
            Some(size) => {
                let n = std::io::copy(&mut r.take(size as _), &mut buf)? as _;
                if size != n {
                    return Err(Error::OutOfBounds);
                }
            }
            None => {
                std::io::copy(r, &mut buf)?;
            }
        };

        Ok(Cursor::new(buf))
    }

    #[cfg(feature = "tokio")]
    pub(crate) async fn read_body_tokio<R: ::tokio::io::AsyncRead + Unpin + ?Sized>(
        &self,
        r: &mut R,
    ) -> Result<Cursor<Vec<u8>>> {
        use ::tokio::io::AsyncReadExt;

        // TODO This allocates on the heap.
        // Ideally, we should use ReadFrom instead of Decode to avoid this.

        // Don't use `with_capacity` on an untrusted size
        // We allocate at most 4096 bytes upfront and grow as needed
        let cap = self.size.unwrap_or(0).min(4096);
        let mut buf = Vec::with_capacity(cap);

        match self.size {
            Some(size) => {
                let n = ::tokio::io::copy(&mut r.take(size as _), &mut buf).await? as _;
                if size != n {
                    return Err(Error::OutOfBounds);
                }
            }
            None => {
                ::tokio::io::copy(r, &mut buf).await?;
            }
        };

        Ok(Cursor::new(buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KIND: FourCC = FourCC::new(b"free");

    #[test]
    fn body_size_subtracts_header() {
        assert_eq!(body_size(KIND, 8, 8).unwrap(), 0);
        assert_eq!(body_size(KIND, 24, 16).unwrap(), 8);
        assert!(matches!(body_size(KIND, 7, 8), Err(Error::InvalidSize)));
        assert!(matches!(body_size(KIND, 15, 16), Err(Error::InvalidSize)));
    }

    #[test]
    #[cfg(target_pointer_width = "32")]
    fn body_size_rejects_usize_overflow() {
        let size = u32::MAX as u64 + 17;
        assert!(matches!(
            body_size(KIND, size, 16),
            Err(Error::TooLarge(KIND))
        ));
    }

    #[test]
    fn decode_large_size() {
        let mut buf: &[u8] = &[0, 0, 0, 1, b'f', b'r', b'e', b'e', 0, 0, 0, 0, 0, 0, 0, 24];
        let header = Header::decode(&mut buf).unwrap();
        assert_eq!(header.size, Some(8));

        let mut buf: &[u8] = &[0, 0, 0, 1, b'f', b'r', b'e', b'e', 0, 0, 0, 0, 0, 0, 0, 15];
        assert!(matches!(Header::decode(&mut buf), Err(Error::InvalidSize)));
    }

    #[test]
    fn read_from_matches_decode() {
        let cases: &[&[u8]] = &[
            &[0, 0, 0, 0, b'f', b'r', b'e', b'e'],
            &[0, 0, 0, 16, b'f', b'r', b'e', b'e'],
            &[0, 0, 0, 1, b'f', b'r', b'e', b'e', 0, 0, 0, 0, 0, 0, 0, 24],
        ];

        for case in cases {
            let decoded = Header::decode(&mut &case[..]).unwrap();
            let read = <Header as ReadFrom>::read_from(&mut &case[..]).unwrap();
            assert_eq!(decoded.kind, read.kind);
            assert_eq!(decoded.size, read.size);
        }

        let empty: &[u8] = &[];
        assert!(<Option<Header> as ReadFrom>::read_from(&mut &empty[..])
            .unwrap()
            .is_none());
    }

    #[test]
    fn encode_size_boundary() {
        for size in [0, u32::MAX as usize - 8] {
            let header = Header {
                kind: KIND,
                size: Some(size),
            };
            let mut buf = Vec::new();
            header.encode(&mut buf).unwrap();
            assert_eq!(Header::decode(&mut &buf[..]).unwrap().size, Some(size));
        }
    }

    #[test]
    #[cfg(target_pointer_width = "32")]
    fn encode_rejects_overflow() {
        let header = Header {
            kind: KIND,
            size: Some((u32::MAX - 7) as usize),
        };
        assert!(matches!(
            header.encode(&mut Vec::new()),
            Err(Error::TooLarge(KIND))
        ));
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn encode_rejects_overflow() {
        let header = Header {
            kind: KIND,
            size: Some(usize::MAX - 15),
        };
        assert!(matches!(
            header.encode(&mut Vec::new()),
            Err(Error::TooLarge(KIND))
        ));
    }
}
