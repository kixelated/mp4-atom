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

        let size = match size {
            0 => None,
            1 => {
                // Read another 8 bytes
                let size = u64::decode(buf)?;
                Some(size.checked_sub(16).ok_or(Error::InvalidSize)? as usize)
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

/// Read exactly `buf.len()` bytes into `buf`, without retrying
/// `ErrorKind::Interrupted`.
///
/// `Read::read_exact` follows the EINTR convention and loops on `Interrupted`.
/// A blocking `Read` that signals a fired cancellation as `Interrupted` cannot
/// then break that loop; propagating it instead lets the caller stop. Partial
/// reads are still assembled, and a genuine short read (`read` returns 0 before
/// `buf` is filled) is `UnexpectedEof`, exactly as `read_exact` reports it.
fn read_exact_no_retry<R: Read + ?Sized>(r: &mut R, mut buf: &mut [u8]) -> std::io::Result<()> {
    while !buf.is_empty() {
        match r.read(buf)? {
            0 => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "failed to fill whole buffer",
                ));
            }
            n => buf = &mut buf[n..],
        }
    }
    Ok(())
}

/// Append up to `limit` bytes (or to EOF when `limit` is `None`) from `r` into
/// `dst`, without retrying `ErrorKind::Interrupted` — the non-retrying twin of
/// `std::io::copy`, for the same reason as [`read_exact_no_retry`]. Returns the
/// number of bytes appended; a short read simply stops, leaving the caller to
/// decide (`read_body` maps a sized short read to `OutOfBounds`).
fn read_into_no_retry<R: Read + ?Sized>(
    r: &mut R,
    dst: &mut Vec<u8>,
    limit: Option<usize>,
) -> std::io::Result<usize> {
    let mut scratch = [0u8; 8192];
    let mut total = 0usize;
    loop {
        let want = match limit {
            Some(limit) => {
                if total >= limit {
                    break;
                }
                (limit - total).min(scratch.len())
            }
            None => scratch.len(),
        };
        match r.read(&mut scratch[..want])? {
            0 => break, // EOF
            n => {
                dst.extend_from_slice(&scratch[..n]);
                total += n;
            }
        }
    }
    Ok(total)
}

impl ReadFrom for Option<Header> {
    fn read_from<R: Read + ?Sized>(r: &mut R) -> Result<Self> {
        let mut buf = [0u8; 8];
        let n = r.read(&mut buf)?;
        if n == 0 {
            return Ok(None);
        }

        read_exact_no_retry(r, &mut buf[n..])?;

        let size = u32::from_be_bytes(buf[0..4].try_into().unwrap());
        let kind = u32::from_be_bytes(buf[4..8].try_into().unwrap()).into();

        let size = match size {
            0 => None,
            1 => {
                // Read another 8 bytes
                read_exact_no_retry(r, &mut buf)?;
                let size = u64::from_be_bytes(buf);
                let size = size.checked_sub(16).ok_or(Error::InvalidSize)?;

                Some(size as usize)
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
                // `std::io::copy` retries `Interrupted`; use the non-retrying pump so
                // a blocking, partial-reading cancel-carrier surfaces the cancel
                // instead of hot-spinning (same reason as `read_exact_no_retry`).
                let n = read_into_no_retry(r, &mut buf, Some(size))?;
                if size != n {
                    return Err(Error::OutOfBounds);
                }
            }
            None => {
                read_into_no_retry(r, &mut buf, None)?;
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
    use std::collections::VecDeque;
    use std::io::{self, Read};

    /// A `Read` that hands back `chunks` one per call (modelling a partial-reading
    /// blocking source), then — once drained — either returns `Ok(0)` (clean EOF)
    /// or the configured trailing error kind on every further `read`.
    struct Chunked {
        chunks: VecDeque<Vec<u8>>,
        trailing: Option<io::ErrorKind>,
    }

    impl Read for Chunked {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            match self.chunks.pop_front() {
                Some(c) => {
                    let n = c.len().min(buf.len());
                    buf[..n].copy_from_slice(&c[..n]);
                    Ok(n)
                }
                None => match self.trailing {
                    Some(kind) => Err(io::Error::new(kind, "chunked: drained")),
                    None => Ok(0),
                },
            }
        }
    }

    fn chunked(chunks: &[&[u8]], trailing: Option<io::ErrorKind>) -> Chunked {
        Chunked {
            chunks: chunks.iter().map(|c| c.to_vec()).collect(),
            trailing,
        }
    }

    #[test]
    fn read_exact_no_retry_propagates_interrupted() {
        // A partial read, then `Interrupted`: the non-retrying helper must SURFACE
        // it, not loop. (A retrying `read_exact` would hot-spin here forever — this
        // test returning at all is the property under test.)
        let mut r = chunked(&[&[0u8; 4]], Some(io::ErrorKind::Interrupted));
        let mut buf = [0u8; 8];
        let err = read_exact_no_retry(&mut r, &mut buf).expect_err("Interrupted must propagate");
        assert_eq!(err.kind(), io::ErrorKind::Interrupted);
    }

    #[test]
    fn read_exact_no_retry_short_read_is_eof() {
        // A partial read then a genuine `Ok(0)` short read stays `UnexpectedEof`,
        // exactly as `read_exact`'s own contract.
        let mut r = chunked(&[&[0u8; 4]], None);
        let mut buf = [0u8; 8];
        let err = read_exact_no_retry(&mut r, &mut buf).expect_err("a short read is EOF");
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn read_exact_no_retry_assembles_partial_reads() {
        // Multiple partial reads are still stitched into the full buffer.
        let mut r = chunked(&[&[1, 2, 3], &[4, 5, 6, 7, 8]], None);
        let mut buf = [0u8; 8];
        read_exact_no_retry(&mut r, &mut buf).expect("partials assemble to a full read");
        assert_eq!(buf, [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn option_header_read_from_propagates_interrupted() {
        // End-to-end through the actual box-header read path: a partial header
        // followed by `Interrupted` surfaces as `Error::Io(Interrupted)` (which the
        // demuxer translates to its cancellation), never a hot-spin.
        let mut r = chunked(&[&[0u8; 4]], Some(io::ErrorKind::Interrupted));
        let err = <Option<Header> as ReadFrom>::read_from(&mut r)
            .expect_err("cancelled header read is an Err");
        match err {
            Error::Io(io) => assert_eq!(io.kind(), io::ErrorKind::Interrupted),
            other => panic!("expected Error::Io(Interrupted), got {other:?}"),
        }
    }

    #[test]
    fn read_body_propagates_interrupted() {
        // A partial body chunk then `Interrupted` must SURFACE, not hot-spin the way
        // `std::io::copy` would.
        let mut r = chunked(&[&[0u8; 4]], Some(io::ErrorKind::Interrupted));
        let header = Header {
            kind: 0u32.into(),
            size: Some(8),
        };
        let err = header
            .read_body(&mut r)
            .expect_err("Interrupted must propagate");
        match err {
            Error::Io(io) => assert_eq!(io.kind(), io::ErrorKind::Interrupted),
            other => panic!("expected Error::Io(Interrupted), got {other:?}"),
        }
    }

    #[test]
    fn read_body_unsized_propagates_interrupted() {
        // The unbounded (`size == None`, read-to-EOF) arm must not retry either.
        let mut r = chunked(&[&[1, 2, 3]], Some(io::ErrorKind::Interrupted));
        let header = Header {
            kind: 0u32.into(),
            size: None,
        };
        let err = header
            .read_body(&mut r)
            .expect_err("Interrupted must propagate on the unsized path");
        match err {
            Error::Io(io) => assert_eq!(io.kind(), io::ErrorKind::Interrupted),
            other => panic!("expected Error::Io(Interrupted), got {other:?}"),
        }
    }

    #[test]
    fn read_body_short_read_is_out_of_bounds() {
        // Fewer bytes than `size` followed by a clean EOF stays `OutOfBounds`,
        // exactly as the `std::io::copy` byte-count check reported it.
        let mut r = chunked(&[&[1, 2, 3]], None);
        let header = Header {
            kind: 0u32.into(),
            size: Some(8),
        };
        let err = header
            .read_body(&mut r)
            .expect_err("a short body is OutOfBounds");
        assert!(
            matches!(err, Error::OutOfBounds),
            "expected OutOfBounds, got {err:?}"
        );
    }

    #[test]
    fn read_body_assembles_exact_size_from_partials() {
        // Multiple partial reads are stitched into exactly `size` bytes.
        let mut r = chunked(&[&[1, 2, 3], &[4, 5], &[6]], None);
        let header = Header {
            kind: 0u32.into(),
            size: Some(6),
        };
        let body = header
            .read_body(&mut r)
            .expect("partials assemble to the body");
        assert_eq!(body.into_inner(), vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn read_body_unsized_reads_to_eof() {
        let mut r = chunked(&[&[1, 2, 3], &[4, 5]], None);
        let header = Header {
            kind: 0u32.into(),
            size: None,
        };
        let body = header.read_body(&mut r).expect("unsized body reads to EOF");
        assert_eq!(body.into_inner(), vec![1, 2, 3, 4, 5]);
    }
}
