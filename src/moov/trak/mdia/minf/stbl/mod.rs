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
mod stz2;
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
pub use stz2::*;
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
    pub stsz: Option<Stsz>,
    pub stz2: Option<Stz2>,
    pub stco: Option<Stco>,
    pub co64: Option<Co64>,
    pub sbgp: Vec<Sbgp>,
    pub sgpd: Vec<Sgpd>,
    pub subs: Vec<Subs>,
    pub saiz: Vec<Saiz>,
    pub saio: Vec<Saio>,
    pub cslg: Option<Cslg>,
}

impl Stbl {
    fn do_validation(&self) -> Result<()> {
        if self.stsz.is_none() && self.stz2.is_none() {
            return Err(Error::MissingContent(
                "one of stsz or stz2 is required in stbl box",
            ));
        }
        if self.stsz.is_some() && self.stz2.is_some() {
            // TODO: some kind of better error
            return Err(Error::UnexpectedBox(Stz2::KIND));
        }
        Ok(())
    }
}

impl Atom for Stbl {
    const KIND: FourCC = FourCC::new(b"stbl");

    nested! {
        required: [ Stsd, Stts, Stsc ],
        optional: [ Ctts, Stss, Stco, Co64, Cslg, Stsz, Stz2 ],
        multiple: [ Sbgp, Sgpd, Subs, Saiz, Saio ],
        post_parse: do_validation,
    }
}
