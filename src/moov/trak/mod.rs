mod edts;
mod mdia;
mod tkhd;

pub use edts::*;
pub use mdia::*;
pub use tkhd::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Trak {
    pub tkhd: Tkhd,
    pub edts: Option<Edts>,
    pub meta: Option<Meta>, // TODO is this suppose to be here?
    pub mdia: Mdia,
}

impl Atom for Trak {
    const KIND: FourCC = FourCC::new(b"trak");

    nested! {
        required: [ Tkhd, Mdia ],
        optional: [ Edts, Meta ],
        multiple: [],
    }
}
