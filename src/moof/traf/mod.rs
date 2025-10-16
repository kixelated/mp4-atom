mod tfdt;
mod tfhd;
mod trun;

pub use tfdt::*;
pub use tfhd::*;
pub use trun::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Traf {
    pub tfhd: Tfhd,
    pub tfdt: Option<Tfdt>,
    pub trun: Vec<Trun>,
    pub sbgp: Vec<Sbgp>,
    pub sgpd: Vec<Sgpd>,
    pub subs: Vec<Subs>,
    pub saiz: Vec<Saiz>,
    pub saio: Vec<Saio>,
    pub meta: Option<Meta>,
    pub udta: Option<Udta>,
    #[cfg(feature = "fault-tolerant")]
    pub unexpected: Vec<Any>,
}

impl Atom for Traf {
    const KIND: FourCC = FourCC::new(b"traf");

    nested! {
        required: [ Tfhd ],
        optional: [ Tfdt, Meta, Udta ],
        multiple: [ Trun, Sbgp, Sgpd, Subs, Saiz, Saio ],
    }
}
