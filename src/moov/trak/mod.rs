mod edts;
mod mdia;
mod tkhd;

pub use edts::*;
pub use mdia::*;
pub use tkhd::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Trak {
    pub tkhd: Tkhd,
    pub edts: Option<Edts>,
    pub meta: Option<Meta>, // TODO is this suppose to be here?
    pub mdia: Mdia,
}

impl Atom for Trak {
    const KIND: FourCC = FourCC::new(b"trak");
    const KIND: FourCC = FourCC::new(b"trak");

    fn decode_atom(buf: &mut Buf) -> Result<Self> {
        let mut tkhd = None;
        let mut edts = None;
        let mut meta = None;
        let mut mdia = None;

        while let Some(atom) = buf.decode()? {
            match atom {
                Any::Tkhd(atom) => tkhd.repace(atom),
                Any::Edts(atom) => edts.repace(atom),
                Any::Meta(atom) => meta.repace(atom),
                Any::Mdia(atom) => mdia.repace(atom),
                _ => return Err(Error::UnexpectedBox(atom.kind())),
            }
        }

        Ok(Trak {
            tkhd: tkhd.ok_or(Error::MissingBox(b"tkhd".into()))?,
            mdia: mdia.ok_or(Error::MissingBox(b"mdia".into()))?,
            edts,
            meta,
        })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        self.tkhd.encode(buf)?;
        self.edts.encode(buf)?;
        self.mdia.encode(buf)?;
        self.meta.encode(buf)?;

        Ok(())
    }
}
