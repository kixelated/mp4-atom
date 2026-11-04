use crate::*;

/// ProtectionSchemeInfoBox (`sinf`), ISO/IEC 14496-12:2022 Sect 8.12.2.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sinf {
    pub frma: Frma,
    pub schm: Option<Schm>,
    pub schi: Option<Schi>,
}

impl Atom for Sinf {
    const KIND: FourCC = FourCC::new(b"sinf");

    nested! {
        required: [ Frma ],
        optional: [ Schm, Schi ],
        multiple: [],
    }
}
