use crate::{header::header_len, Decode, Error, Header, Result};

use super::*;

use tokio::io::{AsyncRead, AsyncReadExt};

impl AsyncReadFrom for Header {
    async fn read_from<R: AsyncRead + Unpin + ?Sized>(r: &mut R) -> Result<Self> {
        <Option<Header> as AsyncReadFrom>::read_from(r)
            .await?
            .ok_or(Error::UnexpectedEof)
    }
}

impl AsyncReadFrom for Option<Header> {
    async fn read_from<R: AsyncRead + Unpin + ?Sized>(r: &mut R) -> Result<Self> {
        let mut buf = [0u8; 16];
        let n = r.read(&mut buf[..8]).await?;
        if n == 0 {
            return Ok(None);
        }

        r.read_exact(&mut buf[n..8]).await?;

        // A size of 1 means the real size follows in another 8 bytes.
        let len = header_len(&buf);
        r.read_exact(&mut buf[8..len]).await?;

        Header::decode(&mut &buf[..len]).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{future::Future, pin::pin, task::*};

    // Reading from a slice never blocks, so a single poll completes the future.
    fn read(mut buf: &[u8]) -> Result<Option<Header>> {
        let fut = pin!(<Option<Header> as AsyncReadFrom>::read_from(&mut buf));
        match fut.poll(&mut Context::from_waker(Waker::noop())) {
            Poll::Ready(res) => res,
            Poll::Pending => unreachable!(),
        }
    }

    #[test]
    fn read_from() {
        assert!(read(&[]).unwrap().is_none());

        let header = read(&[0, 0, 0, 16, b'f', b'r', b'e', b'e'])
            .unwrap()
            .unwrap();
        assert_eq!(header.size, Some(8));

        let header = read(&[0, 0, 0, 1, b'f', b'r', b'e', b'e', 0, 0, 0, 0, 0, 0, 0, 24])
            .unwrap()
            .unwrap();
        assert_eq!(header.size, Some(8));

        let res = read(&[0, 0, 0, 1, b'f', b'r', b'e', b'e', 0, 0, 0, 0, 0, 0, 0, 15]);
        assert!(matches!(res, Err(Error::InvalidSize)));
    }
}
