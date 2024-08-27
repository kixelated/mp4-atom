mod elst;
pub use elst::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Edts {
    pub elst: Option<Elst>,
}

impl Atom for Edts {
    const KIND: FourCC = FourCC::new(b"edts");

    fn decode_atom(buf: &mut Bytes) -> Result<Self> {
        let mut elst = None;

        while let Some(atom) = buf.decode()? {
            match atom {
                Any::Elst(atom) => elst = atom.into(),
                atom => return Err(Error::UnexpectedBox(atom.kind())),
            }
        }

        Ok(Edts { elst })
    }

    fn encode_atom(&self, buf: &mut BytesMut) -> Result<()> {
        self.elst.encode(buf)
    }
}
