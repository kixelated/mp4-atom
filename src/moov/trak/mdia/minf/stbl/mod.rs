mod co64;
mod ctts;
mod stco;
mod stsc;
mod stsd;
mod stss;
mod stsz;
mod stts;

pub use co64::*;
pub use ctts::*;
pub use stco::*;
pub use stsc::*;
pub use stsd::*;
pub use stss::*;
pub use stsz::*;
pub use stts::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Stbl {
    pub stsd: Stsd,
    pub stts: Stts,
    pub ctts: Option<Ctts>,
    pub stss: Option<Stss>,
    pub stsc: Stsc,
    pub stsz: Stsz,
    pub stco: Option<Stco>,
    pub co64: Option<Co64>,
}

impl Atom for Stbl {
    const KIND: FourCC = FourCC::new(b"stbl");

    fn decode_atom(buf: &mut Bytes) -> Result<Self> {
        let mut stsd = None;
        let mut stts = None;
        let mut ctts = None;
        let mut stss = None;
        let mut stsc = None;
        let mut stsz = None;
        let mut stco = None;
        let mut co64 = None;

        while let Some(atom) = buf.decode()? {
            match atom {
                Any::Stsd(atom) => stsd = atom.into(),
                Any::Stts(atom) => stts = atom.into(),
                Any::Ctts(atom) => ctts = atom.into(),
                Any::Stss(atom) => stss = atom.into(),
                Any::Stsc(atom) => stsc = atom.into(),
                Any::Stsz(atom) => stsz = atom.into(),
                Any::Stco(atom) => stco = atom.into(),
                Any::Co64(atom) => co64 = atom.into(),
                atom => return Err(Error::UnexpectedBox(atom.kind())),
            }
        }

        if stco.is_none() && co64.is_none() {
            // stco or co64 is required
            return Err(Error::MissingBox(Stco::KIND));
        }

        Ok(Stbl {
            stsd: stsd.ok_or(Error::MissingBox(Stsd::KIND))?,
            stts: stts.ok_or(Error::MissingBox(Stts::KIND))?,
            ctts,
            stss,
            stsc: stsc.ok_or(Error::MissingBox(Stsc::KIND))?,
            stsz: stsz.ok_or(Error::MissingBox(Stsz::KIND))?,
            stco,
            co64,
        })
    }

    fn encode_atom(&self, buf: &mut BytesMut) -> Result<()> {
        self.stsd.encode(buf)?;
        self.stts.encode(buf)?;
        self.ctts.encode(buf)?;
        self.stss.encode(buf)?;
        self.stsc.encode(buf)?;
        self.stsz.encode(buf)?;
        self.stco.encode(buf)?;
        self.co64.encode(buf)?;

        Ok(())
    }
}
