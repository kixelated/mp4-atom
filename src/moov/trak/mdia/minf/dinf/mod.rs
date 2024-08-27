mod dref;
pub use dref::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Dinf {
    dref: Dref,
}

impl Atom for Dinf {
    const KIND: FourCC = FourCC::new(b"dinf");

    fn decode_atom(buf: &mut Bytes) -> Result<Self> {
        let mut dref = None;

        while let Some(atom) = buf.decode()? {
            match atom {
                Any::Dref(atom) => dref = atom.into(),
                atom => return Err(Error::UnexpectedBox(atom.kind())),
            }
        }

        Ok(Dinf {
            dref: dref.ok_or(Error::MissingBox(Dref::KIND))?,
        })
    }

    fn encode_atom(&self, buf: &mut BytesMut) -> Result<()> {
        self.dref.encode(buf)?;
        Ok(())
    }
}
