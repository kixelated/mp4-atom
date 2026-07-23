use std::io::Cursor;

/// A contiguous buffer of bytes.
// We're not using bytes::Buf because of some strange bugs with take().
pub trait Buf {
    fn remaining(&self) -> usize;
    fn has_remaining(&self) -> bool {
        self.remaining() > 0
    }

    fn slice(&self, size: usize) -> &[u8];
    fn advance(&mut self, n: usize);
}

impl Buf for &[u8] {
    fn remaining(&self) -> usize {
        self.len()
    }

    fn slice(&self, size: usize) -> &[u8] {
        self[..size].as_ref()
    }

    fn advance(&mut self, n: usize) {
        *self = &self[n..];
    }
}

impl<T: AsRef<[u8]>> Buf for Cursor<T> {
    fn remaining(&self) -> usize {
        self.get_ref().as_ref().len() - self.position() as usize
    }

    fn slice(&self, size: usize) -> &[u8] {
        let pos = self.position() as usize;
        self.get_ref().as_ref()[pos..pos + size].as_ref()
    }

    fn advance(&mut self, n: usize) {
        self.set_position(self.position() + n as u64);
    }
}

impl<T: Buf + ?Sized> Buf for &mut T {
    fn remaining(&self) -> usize {
        (**self).remaining()
    }

    fn slice(&self, size: usize) -> &[u8] {
        (**self).slice(size)
    }

    fn advance(&mut self, n: usize) {
        (**self).advance(n);
    }
}

/// Drain trailing padding left after an atom's child boxes.
///
/// Some muxers — QuickTime in particular — append a few bytes (commonly a
/// 4-byte zero "terminator") after the last child box of a container atom or
/// sample entry. Such a remainder is shorter than a box header (8 bytes), so it
/// cannot be a box and can only be padding: drain it instead of failing the
/// whole atom with [`Error::UnderDecode`](crate::Error::UnderDecode), matching
/// ffmpeg, GPAC and other demuxers. A remainder of 8 or more bytes is left
/// untouched so genuine trailing corruption is still reported.
pub(crate) fn skip_trailing_padding<B: Buf>(buf: &mut B) {
    let n = buf.remaining();
    if n > 0 && n < 8 {
        buf.advance(n);
    }
}

#[cfg(feature = "bytes")]
impl Buf for bytes::Bytes {
    fn remaining(&self) -> usize {
        self.len()
    }

    fn slice(&self, size: usize) -> &[u8] {
        &self[..size]
    }

    fn advance(&mut self, n: usize) {
        bytes::Buf::advance(self, n);
    }
}

/// A mutable contiguous buffer of bytes.
// We're not using bytes::BufMut because it doesn't allow seeking backwards (to set the size).
pub trait BufMut {
    // Returns the current length.
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // Append a slice to the buffer
    fn append_slice(&mut self, val: &[u8]);

    // Set a slice at a position in the buffer.
    fn set_slice(&mut self, pos: usize, val: &[u8]);
}

impl BufMut for Vec<u8> {
    fn len(&self) -> usize {
        self.len()
    }

    fn append_slice(&mut self, v: &[u8]) {
        self.extend_from_slice(v);
    }

    fn set_slice(&mut self, pos: usize, val: &[u8]) {
        self[pos..pos + val.len()].copy_from_slice(val);
    }
}

impl<T: BufMut + ?Sized> BufMut for &mut T {
    fn len(&self) -> usize {
        (**self).len()
    }

    fn append_slice(&mut self, v: &[u8]) {
        (**self).append_slice(v);
    }

    fn set_slice(&mut self, pos: usize, val: &[u8]) {
        (**self).set_slice(pos, val);
    }
}

#[cfg(feature = "bytes")]
impl BufMut for bytes::BytesMut {
    fn len(&self) -> usize {
        self.len()
    }

    fn append_slice(&mut self, v: &[u8]) {
        self.extend_from_slice(v);
    }

    fn set_slice(&mut self, pos: usize, val: &[u8]) {
        self[pos..pos + val.len()].copy_from_slice(val);
    }
}
