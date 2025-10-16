mod dref;
pub use dref::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dinf {
    pub dref: Dref,
    #[cfg(feature = "fault-tolerant")]
    pub unexpected: Vec<Any>,
}

impl Atom for Dinf {
    const KIND: FourCC = FourCC::new(b"dinf");

    nested! {
        required: [ Dref ],
        optional: [],
        multiple: [],
    }
}
