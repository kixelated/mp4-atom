mod tfdt;
mod tfhd;
mod trun;

pub use tfdt::*;
pub use tfhd::*;
pub use trun::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Traf {
    pub tfhd: Tfhd,
    pub tfdt: Option<Tfdt>,
    pub trun: Option<Trun>,
}

impl Atom for Traf {
    const KIND: FourCC = FourCC::new(b"traf");

    fn decode_atom(buf: &mut Bytes) -> Result<Self> {
        let mut tfhd = None;
        let mut tfdt = None;
        let mut trun = None;

        while let Some(atom) = buf.decode()? {
            match atom {
                Any::Tfhd(atom) => tfhd = atom.into(),
                Any::Tfdt(atom) => tfdt = atom.into(),
                Any::Trun(atom) => trun = atom.into(),
                atom => return Err(Error::UnexpectedBox(atom.kind())),
            }
        }

        Ok(Traf {
            tfhd: tfhd.ok_or(Error::MissingBox(Tfhd::KIND))?,
            tfdt,
            trun,
        })
    }

    fn encode_atom(&self, buf: &mut BytesMut) -> Result<()> {
        self.tfhd.encode(buf)?;
        self.tfdt.encode(buf)?;
        self.trun.encode(buf)?;

        Ok(())
    }
}
