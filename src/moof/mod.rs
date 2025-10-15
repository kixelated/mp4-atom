mod mfhd;
mod traf;

pub use mfhd::*;
pub use traf::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Moof {
    pub mfhd: Mfhd,
    pub traf: Vec<Traf>,
    pub unexpected: Vec<Any>,
}

impl Atom for Moof {
    const KIND: FourCC = FourCC::new(b"moof");

    nested! {
        required: [ Mfhd ],
        optional: [],
        multiple: [ Traf ],
    }
}
