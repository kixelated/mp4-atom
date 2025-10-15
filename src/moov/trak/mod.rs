mod edts;
mod mdia;
mod tkhd;

pub use edts::*;
pub use mdia::*;
pub use tkhd::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Trak {
    pub tkhd: Tkhd,
    pub edts: Option<Edts>,
    pub meta: Option<Meta>, // TODO is this suppose to be here?
    pub mdia: Mdia,
    pub udta: Option<Udta>,
    pub unexpected: Vec<Any>,
}

impl Atom for Trak {
    const KIND: FourCC = FourCC::new(b"trak");

    nested! {
        required: [ Tkhd, Mdia ],
        optional: [ Edts, Meta, Udta ],
        multiple: [],
    }
}
