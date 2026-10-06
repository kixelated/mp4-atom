mod tfdt;
mod tfhd;
mod trun;

pub use tfdt::*;
pub use tfhd::*;
pub use trun::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
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
    pub senc: Option<Senc>,
    pub udta: Option<Udta>,
}

impl Atom for Traf {
    const KIND: FourCC = FourCC::new(b"traf");

    nested! {
        required: [ Tfhd ],
        optional: [ Tfdt, Meta, Senc, Udta ],
        multiple: [ Trun, Sbgp, Sgpd, Subs, Saiz, Saio ],
    }
}
