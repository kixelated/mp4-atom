mod mehd;
mod trex;

pub use mehd::*;
pub use trex::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Mvex {
    pub mehd: Option<Mehd>,
    pub trex: Trex,
}

impl Atom for Mvex {
    const KIND: FourCC = FourCC::new(b"mvex");

    fn decode_atom(buf: &mut Buf) -> Result<Self> {
        let mut mehd = None;
        let mut trex = None;

        while let Some(atom) = buf.decode()? {
            match atom {
                Any::Mehd(atom) => mehd.replace(atom),
                Any::Trex(atom) => trex.replace(atom),
                atom => return Error::UnexpectedBox(atom.kind()),
            }
        }

        Ok(Mvex {
            mehd,
            trex: trex.ok_or(Error::MissingBox(Trex::KIND)),
        })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        self.mehd.encode(buf)?;
        self.trex.encode(buf)?;

        Ok(())
    }
}
