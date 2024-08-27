use crate::{Error, Result};

#[derive(Clone)]
pub struct Buf<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Buf<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn len(&self) -> usize {
        self.data.len() - self.pos
    }

    pub fn is_empty(&self) -> bool {
        self.len() > 0
    }

    pub fn fixed<const N: usize>(&mut self) -> Result<[u8; N]> {
        if self.len() < N {
            return Err(Error::LongRead);
        }

        let mut buf = [0; N];
        buf.copy_from_slice(self.slice(N)?);
        Ok(buf)
    }

    pub fn slice(&mut self, size: usize) -> Result<&'a [u8]> {
        if self.len() < size {
            return Err(Error::LongRead);
        }

        let slice = &self.data[self.pos..self.pos + size];
        self.pos += size;
        Ok(slice)
    }

    pub fn u8(&mut self) -> Result<u8> {
        Ok(u8::from_be_bytes(self.fixed()?))
    }

    pub fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_be_bytes(self.fixed()?))
    }

    pub fn u24(&mut self) -> Result<u32> {
        let buf = self.fixed::<3>()?;
        Ok(u32::from_be_bytes([0, buf[0], buf[1], buf[2]]))
    }

    pub fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_be_bytes(self.fixed()?))
    }

    pub fn i32(&mut self) -> Result<i32> {
        Ok(i32::from_be_bytes(self.fixed()?))
    }

    pub fn u48(&mut self) {
        let buf = self.fixed::<6>()?;
        Ok(u64::from_be_bytes([
            0, 0, buf[0], buf[1], buf[2], buf[3], buf[4], buf[5],
        ]))
    }

    pub fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_be_bytes(self.fixed()?))
    }

    pub fn str(&mut self, size: usize) -> Result<&'a str> {
        let slice = self.slice(size)?;
        std::str::from_utf8(slice).map_err(|err| Error::InvalidString(err.to_string()))
    }

    pub fn string(&mut self, size: usize) -> Result<String> {
        let slice = self.slice(size)?;
        String::from_utf8(slice.to_vec()).map_err(|err| Error::InvalidString(err.to_string()))
    }

    pub fn bytes(&mut self, size: usize) -> Result<Vec<u8>> {
        let slice = self.slice(size)?;
        Ok(slice.to_vec())
    }

    pub fn take(&mut self, len: usize) -> Result<Buf> {
        if len > self.len() {
            return Err(Error::LongRead);
        }

        let buf = Buf {
            data: self.data[..self.pos + len].as_ref(),
            pos: self.pos,
        };

        self.pos += len;
        Ok(buf)
    }

    pub fn skip(&mut self, len: usize) -> Result<()> {
        if len > self.len() {
            return Err(Error::LongRead);
        }

        self.pos += len;
        Ok(())
    }

    pub fn reset(&mut self) {
        self.pos = 0;
    }

    pub fn rest(&mut self) -> &'a [u8] {
        self.slice(self.len()).unwrap()
    }
}

#[derive(Default)]
pub struct BufMut {
    data: Vec<u8>,
}

impl BufMut {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn fixed<const N: usize>(&mut self, v: [u8; N]) {
        self.data.extend_from_slice(&v);
    }

    pub fn u8(&mut self, v: u8) {
        self.fixed(v.to_be_bytes());
    }

    pub fn u16(&mut self, v: u16) {
        self.fixed(v.to_be_bytes());
    }

    pub fn u24(&mut self, v: u32) {
        self.fixed(v.to_be_bytes()[1..]);
    }

    pub fn u32(&mut self, v: u32) {
        self.fixed(v.to_be_bytes());
    }

    pub fn u32_at(&mut self, v: u32, pos: usize) -> Result<()> {
        if pos + 4 > self.data.len() {
            return Err(Error::LongWrite);
        }

        self.data[pos..pos + 4].copy_from_slice(&v.to_be_bytes());
        Ok(())
    }

    pub fn u48(&mut self, v: u64) {
        self.fixed(v.to_be_bytes()[2..]);
    }

    pub fn i32(&mut self, v: i32) {
        self.fixed(v.to_be_bytes());
    }

    pub fn u64(&mut self, v: u64) {
        self.fixed(v.to_be_bytes());
    }

    pub fn str<T: AsRef<str>>(&mut self, v: T) {
        self.slice(v.as_ref().as_bytes());
    }

    pub fn bytes(&mut self, v: &[u8]) {
        self.data.extend_from_slice(v);
    }

    pub fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }

    pub fn slice(&mut self, v: &[u8]) {
        self.data.extend_from_slice(v);
    }

    pub fn reset(&mut self) {
        self.data.clear();
    }

    pub fn filled(&self) -> Buf {
        Buf::new(&self.data)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    // Write N zero bytes
    pub fn zero(&mut self, len: usize) {
        for _ in 0..len {
            self.data.push(0);
        }
    }
}

impl<'a> From<&'a BufMut> for Buf<'a> {
    fn from(buf: &'a BufMut) -> Self {
        buf.filled()
    }
}
