pub use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Url {
    pub location: String,
}

impl AtomExt for Url {
    type Ext = ();

    const KIND: FourCC = FourCC::new(b"url ");

    fn decode_atom(buf: &mut Buf, _ext: ()) -> Result<Self> {
        Ok(Url {
            location: buf.decode()?,
        })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        self.location.encode(buf)?;
        Ok(())
    }
}
