mod mvex;
mod mvhd;
mod trak;
mod udta;

pub use mvex::*;
pub use mvhd::*;
pub use trak::*;
pub use udta::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Moov {
    pub mvhd: Mvhd,
    pub meta: Option<Meta>,
    pub mvex: Option<Mvex>,
    pub trak: Vec<Trak>,
    pub udta: Option<Udta>,
}

impl Atom for Moov {
    const KIND: FourCC = FourCC::new(b"moov");

    nested! {
        required: [ Mvhd ],
        optional: [ Meta, Mvex, Udta ],
        multiple: [ Trak ],
    }
}
