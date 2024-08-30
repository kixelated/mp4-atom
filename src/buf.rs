use crate::*;

/*
pub struct Buf<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Buf<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    pub fn slice(&mut self, len: usize) -> Result<&'a [u8]> {
        if self.pos + len > self.data.len() {
            return Err(Error::OutOfBounds);
        }

        let slice = &self.data[self.pos..self.pos + len];
        self.pos += len;

        Ok(slice)
    }

    pub fn fixed<const N: usize>(&mut self) -> Result<[u8; N]> {
        Ok(self.slice(N)?.try_into().unwrap())
    }
}
    */

/*
#[derive(Default)]
pub struct BufMut {
    data: Vec<u8>,
}

impl BufMut {
    pub fn new() -> Self {
        Self {}
    }

    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    pub fn fixed<const N: usize>(&mut self, v: [u8; N]) -> Result<()> {
        Ok(self.slice(N)?.try_into().unwrap())
    }
}
*/
