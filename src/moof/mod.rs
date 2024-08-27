mod mfhd;
mod traf;

pub use mfhd::*;
pub use traf::*;

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Moof {
    pub mfhd: Mfhd,
    pub trafs: Vec<Traf>,
}

impl Atom for Moof {
    const KIND: FourCC = FourCC::new(b"moof");

    fn decode_atom(buf: &mut Bytes) -> Result<Self> {
        let mut mfhd = None;
        let mut trafs = Vec::new();

        while let Some(atom) = buf.decode()? {
            match atom {
                Any::Mfhd(atom) => mfhd = atom.into(),
                Any::Traf(atom) => trafs.push(atom),
                other => return Err(Error::UnexpectedBox(other.kind())),
            }
        }

        Ok(Moof {
            mfhd: mfhd.ok_or(Error::MissingBox(Mfhd::KIND))?,
            trafs,
        })
    }

    fn encode_atom(&self, buf: &mut BytesMut) -> Result<()> {
        self.mfhd.encode(buf)?;
        for traf in &self.trafs {
            traf.encode(buf)?;
        }
        Ok(())
    }
}
