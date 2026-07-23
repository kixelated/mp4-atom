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

/// Backfill the size field of a box that was encoded with an 8-byte header
/// placeholder (a 4-byte size of `0` followed by the 4-byte kind) starting at
/// `start`. `len` is the number of bytes written since `start` (header
/// placeholder + body).
///
/// If the box fits in 32 bits the placeholder is simply overwritten. Otherwise
/// the 64-bit `largesize` form is used: the size field becomes `1` and an 8-byte
/// largesize is spliced in after the kind, growing the header to 16 bytes.
pub(crate) fn write_box_size<B: BufMut>(
    buf: &mut B,
    start: usize,
    len: usize,
    kind: FourCC,
) -> Result<()> {
    match u32::try_from(len) {
        Ok(size) => {
            buf.set_slice(start, &size.to_be_bytes());
        }
        Err(_) => {
            // The 16-byte largesize header replaces the 8-byte one, so the total
            // box size is the current length plus the extra 8 bytes.
            let total = (len as u64).checked_add(8).ok_or(Error::TooLarge(kind))?;
            buf.insert_slice(start + 8, &total.to_be_bytes());
            buf.set_slice(start, &1u32.to_be_bytes());
        }
    }
    Ok(())
}

impl Encode for Header {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        match self.size {
            None => {
                0u32.encode(buf)?;
                self.kind.encode(buf)
            }
            Some(size) => {
                // Total box size including the 8-byte basic header.
                let total = size.checked_add(8).ok_or(Error::TooLarge(self.kind))?;
                if total <= u32::MAX as usize {
                    (total as u32).encode(buf)?;
                    self.kind.encode(buf)
                } else {
                    // Use the 64-bit largesize form: size==1, then an 8-byte
                    // largesize covering the full 16-byte header plus the body.
                    // (Only reachable on 64-bit targets, since a >4 GiB buffer
                    // cannot exist on a 32-bit target.)
                    1u32.encode(buf)?;
                    self.kind.encode(buf)?;
                    let large = (size as u64)
                        .checked_add(16)
                        .ok_or(Error::TooLarge(self.kind))?;
                    large.encode(buf)
                }
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
            1 => {
                // Read another 8 bytes
                let size = u64::decode(buf)?;
                let size = size.checked_sub(16).ok_or(Error::InvalidSize)?;
                // On 32-bit targets a `as usize` cast would silently truncate,
                // producing a misaligned parse instead of a clean error.
                Some(usize::try_from(size).map_err(|_| Error::TooLarge(kind))?)
            }
            _ => Some(size.checked_sub(8).ok_or(Error::InvalidSize)? as usize),
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

                // Avoid silent truncation of a 64-bit size on 32-bit targets.
                Some(usize::try_from(size).map_err(|_| Error::TooLarge(kind))?)
            }
            _ => Some(size.checked_sub(8).ok_or(Error::InvalidSize)? as usize),
        };

        Ok(Some(Header { kind, size }))
    }
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

    #[test]
    fn header_largesize_round_trip() {
        // A body just over u32::MAX must use the 64-bit largesize form. We only
        // exercise the header itself (no body) to avoid a multi-GiB allocation.
        let body_size = u32::MAX as usize + 1;
        let header = Header {
            kind: FourCC::new(b"mdat"),
            size: Some(body_size),
        };

        let mut buf = Vec::new();
        header.encode(&mut buf).unwrap();

        // size==1 marker, kind, then an 8-byte largesize == body + 16-byte header.
        assert_eq!(&buf[0..4], &1u32.to_be_bytes());
        assert_eq!(&buf[4..8], b"mdat");
        assert_eq!(
            u64::from_be_bytes(buf[8..16].try_into().unwrap()),
            body_size as u64 + 16
        );

        let decoded = Header::decode(&mut buf.as_slice()).unwrap();
        assert_eq!(decoded.kind, header.kind);
        assert_eq!(decoded.size, Some(body_size));
    }

    #[test]
    fn write_box_size_switches_to_largesize() {
        // Simulate a box whose encoded length just exceeds u32::MAX without
        // actually allocating it: start with the 8-byte header placeholder and
        // claim a large length.
        let mut buf = vec![0u8; 8];
        buf[4..8].copy_from_slice(b"mdat");
        let len = u32::MAX as usize + 1; // pretend the body made us this big

        write_box_size(&mut buf, 0, len, FourCC::new(b"mdat")).unwrap();

        // 8 bytes should have been inserted after the kind.
        assert_eq!(buf.len(), 16);
        assert_eq!(&buf[0..4], &1u32.to_be_bytes());
        assert_eq!(&buf[4..8], b"mdat");
        assert_eq!(
            u64::from_be_bytes(buf[8..16].try_into().unwrap()),
            len as u64 + 8
        );
    }
}
