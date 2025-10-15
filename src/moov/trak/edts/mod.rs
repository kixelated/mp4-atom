mod elst;
pub use elst::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Edts {
    pub elst: Option<Elst>,
    pub unexpected: Vec<Any>,
}

impl Atom for Edts {
    const KIND: FourCC = FourCC::new(b"edts");

    nested! {
        required: [],
        optional: [ Elst ],
        multiple: [],
    }
}
