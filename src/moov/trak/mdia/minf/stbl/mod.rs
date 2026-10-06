mod co64;
mod cslg;
mod ctts;
mod saiz;
mod sbgp;
mod sgpd;
mod stco;
mod stsc;
mod stsd;
mod stss;
mod stsz;
mod stts;
mod subs;

pub use co64::*;
pub use cslg::*;
pub use ctts::*;
pub use saiz::*;
pub use sbgp::*;
pub use sgpd::*;
pub use stco::*;
pub use stsc::*;
pub use stsd::*;
pub use stss::*;
pub use stsz::*;
pub use stts::*;
pub use subs::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Stbl {
    pub stsd: Stsd,
    pub stts: Stts,
    pub ctts: Option<Ctts>,
    pub stss: Option<Stss>,
    pub stsc: Stsc,
    pub stsz: Stsz,
    pub stco: Option<Stco>,
    pub co64: Option<Co64>,
    pub sbgp: Vec<Sbgp>,
    pub sgpd: Vec<Sgpd>,
    pub subs: Vec<Subs>,
    pub saiz: Vec<Saiz>,
    pub saio: Vec<Saio>,
    pub cslg: Option<Cslg>,
}

impl Atom for Stbl {
    const KIND: FourCC = FourCC::new(b"stbl");

    nested! {
        required: [ Stsd, Stts, Stsc, Stsz ],
        optional: [ Ctts, Stss, Stco, Co64, Cslg ],
        multiple: [ Sbgp, Sgpd, Subs, Saiz, Saio ],
    }
}
