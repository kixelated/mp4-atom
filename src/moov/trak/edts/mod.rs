mod elst;
pub use elst::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Edts {
    pub elst: Option<Elst>,
}

impl Atom for Edts {
    const KIND: FourCC = FourCC::new(b"edts");

    nested! {
        required: [],
        optional: [ Elst ],
        multiple: [],
    }
}
