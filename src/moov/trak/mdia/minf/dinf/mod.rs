mod dref;
pub use dref::*;


#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Dinf {
    pub dref: Dref,
}

impl Atom for Dinf {
    const KIND: FourCC = FourCC::new(b"dinf");

    nested! {
        required: [ Dref ],
        optional: [],
        multiple: [],
    }
}
