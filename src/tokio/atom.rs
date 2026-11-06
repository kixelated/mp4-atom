use super::*;

use crate::{Atom, Buf, DecodeAtom, Encode, Error, Header, Result};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

impl<T: Encode> AsyncWriteTo for T {
    async fn write_to<W: AsyncWrite + Unpin + ?Sized>(&self, w: &mut W) -> Result<()> {
        // TODO We should avoid allocating a buffer here.
        let mut buf = Vec::new();
        self.encode(&mut buf)?;
        Ok(w.write_all(&buf).await?)
    }
}

impl<T: Atom> AsyncReadFrom for T {
    async fn read_from<R: AsyncRead + Unpin + ?Sized>(r: &mut R) -> Result<Self> {
        <Option<T> as AsyncReadFrom>::read_from(r)
            .await?
            .ok_or(Error::MissingBox(T::KIND))
    }
}

impl<T: Atom> AsyncReadFrom for Option<T> {
    async fn read_from<R: AsyncRead + Unpin + ?Sized>(r: &mut R) -> Result<Self> {
        let header = match Option::<Header>::read_from(r).await? {
            Some(header) => header,
            None => return Ok(None),
        };

        if header.kind != T::KIND {
            return Err(Error::UnexpectedBox(header.kind));
        }

        let mut buf = header.read_body_tokio(r).await?;

        let atom = match T::decode_body(&mut buf) {
            Ok(atom) => atom,
            Err(Error::OutOfBounds) => return Err(Error::OverDecode(T::KIND)),
            Err(Error::ShortRead) => return Err(Error::UnderDecode(T::KIND)),
            Err(err) => return Err(err),
        };

        if buf.has_remaining() {
            return Err(Error::UnderDecode(T::KIND));
        }

        Ok(Some(atom))
    }
}

impl<T: Atom> AsyncReadUntil for T {
    async fn read_until<R: AsyncRead + Unpin + ?Sized>(r: &mut R) -> Result<Self> {
        Option::<T>::read_until(r)
            .await?
            .ok_or(Error::MissingBox(T::KIND))
    }
}

impl<T: Atom> AsyncReadUntil for Option<T> {
    async fn read_until<R: AsyncRead + Unpin + ?Sized>(r: &mut R) -> Result<Self> {
        while let Some(header) = Option::<Header>::read_from(r).await? {
            if header.kind == T::KIND {
                let mut buf = header.read_body_tokio(r).await?;
                return Ok(Some(T::decode_atom(&header, &mut buf)?));
            }

            // Skip the body without buffering it, so the next header is read
            // from after this box rather than from inside it.
            match header.size {
                Some(size) => {
                    let n =
                        tokio::io::copy(&mut r.take(size as u64), &mut tokio::io::sink()).await?;
                    if n != size as u64 {
                        return Err(Error::OutOfBounds);
                    }
                }
                // The box extends to the end of the input, so nothing follows it.
                None => return Ok(None),
            }
        }

        Ok(None)
    }
}

impl<T: Atom> AsyncReadAtom for T {
    async fn read_atom<R: AsyncRead + Unpin + ?Sized>(header: &Header, r: &mut R) -> Result<Self> {
        if header.kind != T::KIND {
            return Err(Error::UnexpectedBox(header.kind));
        }

        let mut buf = header.read_body_tokio(r).await?;
        Self::decode_atom(header, &mut buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Free, Ftyp, Zeroed};

    use std::future::Future;
    use std::task::{Context, Poll, Waker};

    // In-memory readers never pend, so a single poll completes the future.
    fn block_on<F: Future>(fut: F) -> F::Output {
        let mut fut = std::pin::pin!(fut);
        match fut.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
            Poll::Ready(output) => output,
            Poll::Pending => panic!("in-memory read should not pend"),
        }
    }

    fn ftyp() -> Ftyp {
        Ftyp {
            major_brand: b"iso6".into(),
            minor_version: 512,
            compatible_brands: vec![b"mp41".into()],
        }
    }

    fn encode(atom: &impl Encode) -> Vec<u8> {
        let mut buf = Vec::new();
        atom.encode(&mut buf).unwrap();
        buf
    }

    #[test]
    fn read_until_ignores_box_nested_in_skipped_body() {
        // A free box whose body happens to contain a valid ftyp.
        let forged = encode(&ftyp());
        let mut input = (8 + forged.len() as u32).to_be_bytes().to_vec();
        input.extend_from_slice(b"free");
        input.extend_from_slice(&forged);

        let found = block_on(Option::<Ftyp>::read_until(&mut input.as_slice())).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn read_until_finds_box_after_skipped_box() {
        let mut input = encode(&Free {
            zeroed: Zeroed::new(16),
        });
        input.extend(encode(&ftyp()));

        let found = block_on(Ftyp::read_until(&mut input.as_slice())).unwrap();
        assert_eq!(found, ftyp());
    }

    #[test]
    fn read_until_rejects_truncated_skipped_box() {
        let mut input = encode(&Free {
            zeroed: Zeroed::new(16),
        });
        input.truncate(input.len() - 4);

        let err = block_on(Ftyp::read_until(&mut input.as_slice())).unwrap_err();
        assert!(matches!(err, Error::OutOfBounds));
    }
}
