mod mfhd;
mod traf;

pub use mfhd::*;
pub use traf::*;

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Moof {
    pub mfhd: Mfhd,
    pub traf: Vec<Traf>,
}

impl Atom for Moof {
    const KIND: FourCC = FourCC::new(b"moof");

    nested! {
        required: [ Mfhd ],
        optional: [],
        multiple: [ Traf ],
    }
}
