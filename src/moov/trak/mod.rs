mod edts;
mod mdia;
mod senc;
mod tkhd;

pub use edts::*;
pub use mdia::*;
pub use senc::*;
pub use tkhd::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Trak {
    pub tkhd: Tkhd,
    pub edts: Option<Edts>,
    pub meta: Option<Meta>, // TODO is this suppose to be here?
    pub mdia: Mdia,
    pub senc: Option<Senc>,
    pub udta: Option<Udta>,
}

impl Atom for Trak {
    const KIND: FourCC = FourCC::new(b"trak");

    nested! {
        required: [ Tkhd, Mdia ],
        optional: [ Edts, Meta, Senc, Udta ],
        multiple: [],
    }
}
