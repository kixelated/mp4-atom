mod hdlr;
mod mdhd;
mod minf;

pub use hdlr::*;
pub use mdhd::*;
pub use minf::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mdia {
    pub mdhd: Mdhd,
    pub hdlr: Hdlr,
    pub minf: Minf,
    #[cfg(feature = "fault-tolerant")]
    pub unexpected: Vec<Any>,
}

impl Atom for Mdia {
    const KIND: FourCC = FourCC::new(b"mdia");

    nested! {
        required: [ Mdhd, Hdlr, Minf ],
        optional: [] ,
        multiple: [],
    }
}
