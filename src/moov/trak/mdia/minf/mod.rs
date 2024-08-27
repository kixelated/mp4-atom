mod dinf;
mod smhd;
mod stbl;
mod vmhd;

pub use dinf::*;
pub use smhd::*;
pub use stbl::*;
pub use vmhd::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Minf {
    pub vmhd: Option<Vmhd>,
    pub smhd: Option<Smhd>,
    pub dinf: Dinf,
    pub stbl: Stbl,
}

impl Atom for Minf {
    const KIND: FourCC = FourCC::new(b"minf");

    fn decode_atom(buf: &mut Bytes) -> Result<Self> {
        let mut vmhd = None;
        let mut smhd = None;
        let mut dinf = None;
        let mut stbl = None;

        while let Some(atom) = buf.decode()? {
            match atom {
                Any::Vmhd(atom) => vmhd = atom.into(),
                Any::Smhd(atom) => smhd = atom.into(),
                Any::Dinf(atom) => dinf = atom.into(),
                Any::Stbl(atom) => stbl = atom.into(),
                atom => return Err(Error::UnexpectedBox(atom.kind())),
            }
        }

        Ok(Minf {
            vmhd,
            smhd,
            dinf: dinf.ok_or(Error::MissingBox(Dinf::KIND))?,
            stbl: stbl.ok_or(Error::MissingBox(Stbl::KIND))?,
        })
    }

    fn encode_atom(&self, buf: &mut BytesMut) -> Result<()> {
        self.vmhd.encode(buf)?;
        self.smhd.encode(buf)?;
        self.dinf.encode(buf)?;
        self.stbl.encode(buf)?;

        Ok(())
    }
}
