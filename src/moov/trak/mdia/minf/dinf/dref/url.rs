pub use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Url {
    pub location: String,
}

impl AtomExt for Url {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"url ");

    fn decode_atom_ext(buf: &mut Bytes, _ext: ()) -> Result<Self> {
        Ok(Url {
            location: buf.decode()?,
        })
    }

    fn encode_atom_ext(&self, buf: &mut BytesMut) -> Result<()> {
        self.location.as_str().encode(buf)?;
        Ok(())
    }
}
