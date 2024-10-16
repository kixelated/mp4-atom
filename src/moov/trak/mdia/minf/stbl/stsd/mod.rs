mod avc1;
mod hev1;
mod mp4a;
mod tx3g;
mod vp09;
mod vpcc;

pub use avc1::*;
pub use hev1::*;
pub use mp4a::*;
pub use tx3g::*;
pub use vp09::*;
pub use vpcc::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Stsd {
    pub avc1: Option<Avc1>,
    pub hev1: Option<Hev1>,
    pub vp09: Option<Vp09>,
    pub mp4a: Option<Mp4a>,
    pub tx3g: Option<Tx3g>,
}

impl AtomExt for Stsd {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"stsd");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let entries = u32::decode(buf)?;

        let mut avc1 = None;
        let mut hev1 = None;
        let mut vp09 = None;
        let mut mp4a = None;
        let mut tx3g = None;

        for _ in 0..entries {
            let atom = Any::decode(buf)?;
            match atom {
                Any::Avc1(atom) => avc1 = atom.into(),
                Any::Hev1(atom) => hev1 = atom.into(),
                Any::Vp09(atom) => vp09 = atom.into(),
                Any::Mp4a(atom) => mp4a = atom.into(),
                Any::Tx3g(atom) => tx3g = atom.into(),
                Any::Unknown(kind, _) => tracing::warn!("unknown atom: {:?}", kind),
                _ => return Err(Error::UnexpectedBox(atom.kind())),
            }
        }

        Ok(Stsd {
            avc1,
            hev1,
            vp09,
            mp4a,
            tx3g,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        (self.avc1.is_some() as u32
            + self.hev1.is_some() as u32
            + self.vp09.is_some() as u32
            + self.mp4a.is_some() as u32
            + self.tx3g.is_some() as u32)
            .encode(buf)?;

        self.avc1.encode(buf)?;
        self.hev1.encode(buf)?;
        self.vp09.encode(buf)?;
        self.mp4a.encode(buf)?;
        self.tx3g.encode(buf)?;

        Ok(())
    }
}
