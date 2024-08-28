use crate::*;
use std::io::{self, Read};

/// Used to read top-level boxes
pub struct Reader<R: Read> {
    input: R,
    buf: BytesMut,
}

impl<R: Read> Reader<R> {
    /// Create a new reader
    pub fn new(input: R) -> Self {
        Self {
            input,
            buf: BytesMut::new(),
        }
    }

    // Read the next atom from the input
    pub fn atom(&mut self) -> Result<Option<Any>> {
        let header = match self.header_inner()? {
            Some(header) => header,
            None => return Ok(None),
        };

        let buf = self.fill(header.size)?;
        let mut buf = buf.split_to(header.size.unwrap_or(buf.len())).freeze();

        let atom = Any::decode_with(header, &mut buf)?;
        if buf.len() > 0 {
            return Err(Error::ShortRead);
        }

        Ok(Some(atom))
    }

    // Read the header from the input
    pub fn header(mut self) -> Result<Option<ReaderHeader<R>>> {
        Ok(self.header_inner()?.map(|header| ReaderHeader {
            reader: self,
            header,
        }))
    }

    fn header_inner(&mut self) -> Result<Option<Header>> {
        // Read up to 8 bytes or until EOF.
        let buf = self.read(8)?;
        match buf.len() {
            0 => return Ok(None),
            0..=7 => return Err(Error::ShortRead),
            _ => {}
        }

        let size = u32::from_be_bytes(buf[0..4].try_into().unwrap()) as usize;
        let mut buf = match size {
            // We need another 8 bytes
            1 => self.fill(16.into())?.split_to(16).freeze(),
            _ => self.buf.split_to(8).freeze(),
        };

        Ok(Some(Header::decode(&mut buf)?))
    }

    fn fill(&mut self, size: Option<usize>) -> Result<&mut BytesMut> {
        let buf = self.read(size.unwrap_or(usize::MAX))?;
        if buf.len() < size.unwrap_or(0) {
            return Err(Error::ShortRead);
        }

        Ok(buf)
    }

    fn read(&mut self, limit: usize) -> Result<&mut BytesMut> {
        let limit = limit.saturating_sub(self.buf.len());

        let mut reader = (&mut self.input).take(limit as _);
        let mut writer = self.buf.split_off(0).writer();

        io::copy(&mut reader, &mut writer)?;
        self.buf = writer.into_inner();

        Ok(&mut self.buf)
    }

    fn discard(&mut self, size: Option<usize>) -> Result<()> {
        match size {
            Some(size) => {
                let taken = self.buf.len().min(size);
                self.buf.advance(taken);

                let limit = (size - taken) as u64;
                let mut reader = (&mut self.input).take(limit);
                let actual = io::copy(&mut reader, &mut io::sink())?;

                if actual < limit {
                    return Err(Error::ShortRead);
                }
            }
            None => {
                self.buf.clear();
                io::copy(&mut self.input, &mut io::sink())?;
            }
        };

        Ok(())
    }
}

pub struct ReaderHeader<R: Read> {
    reader: Reader<R>,
    header: Header,
}

impl<R: Read> ReaderHeader<R> {
    pub fn kind(&self) -> FourCC {
        self.header.kind
    }

    /// None means it extends to the end of the file
    pub fn size(&self) -> Option<usize> {
        self.header.size
    }

    pub fn atom(mut self) -> Result<(Any, Reader<R>)> {
        let mut buf = match self.header.size {
            Some(size) => self.reader.fill(Some(size))?.split_to(size).freeze(),
            None => self.reader.fill(None)?.split().freeze(),
        };

        let atom = Any::decode_with(self.header, &mut buf)?;
        if buf.len() > 0 {
            return Err(Error::ShortRead);
        }

        Ok((atom, self.reader))
    }

    pub fn decode<A: Atom>(mut self) -> Result<(A, Reader<R>)> {
        let mut buf = match self.header.size {
            Some(size) => self.reader.fill(Some(size))?.split_to(size).freeze(),
            None => self.reader.fill(None)?.split().freeze(),
        };

        let atom = A::decode_atom(&mut buf)?;
        if buf.len() > 0 {
            return Err(Error::ShortRead);
        }

        Ok((atom, self.reader))
    }

    pub fn raw(self) -> ReaderRaw<R> {
        ReaderRaw {
            reader: self.reader,
            remain: self.header.size,
        }
    }

    pub fn skip(mut self) -> Result<Reader<R>> {
        // Discard the atom
        self.reader.discard(self.header.size)?;
        Ok(self.reader)
    }
}

pub struct ReaderRaw<R: Read> {
    reader: Reader<R>,
    remain: Option<usize>,
}

impl<R: Read> ReaderRaw<R> {
    pub fn skip(mut self) -> Result<Reader<R>> {
        self.reader.discard(self.remain)?;
        Ok(self.reader)
    }
}

impl<R: Read> Read for ReaderRaw<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let size = buf.len().min(self.remain.unwrap_or(usize::MAX));
        let n = self.reader.input.read(&mut buf[..size])?;

        if let Some(remain) = self.remain.as_mut() {
            *remain -= n;
        }

        Ok(n)
    }
}
