mod hdlr;
mod mdhd;
mod minf;

pub use hdlr::*;
pub use mdhd::*;
pub use minf::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Mdia {
    pub mdhd: Mdhd,
    pub hdlr: Hdlr,
    pub minf: Minf,
}

impl Atom for Mdia {
    const KIND: FourCC = FourCC::new(b"mdia");
    fn decode_atom(buf: &mut Buf) -> Result<Self> {
        let mut mdhd = None;
        let mut hdlr = None;
        let mut minf = None;

        while let Some(atom) = buf.decode()? {
            match atom {
                Any::Mdhd(atom) => mdhd.replace(atom),
                Any::Hdlr(atom) => hdlr.replace(atom),
                Any::Minf(atom) => minf.replace(atom),
                atom => return Error::UnexpectedBox(atom.kind()),
            }
        }

        Ok(Mdia {
            mdhd: mdhd.ok_or(Error::MissingBox(Mdhd::KIND)),
            hdlr: hdlr.ok_or(Error::MissingBox(Hdlr::KIND)),
            minf: minf.ok_or(Error::MissingBox(Minf::KIND)),
        })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        self.mdhd.encode(buf)?;
        self.hdlr.encode(buf)?;
        self.minf.encode(buf)?;

        Ok(())
    }
}
