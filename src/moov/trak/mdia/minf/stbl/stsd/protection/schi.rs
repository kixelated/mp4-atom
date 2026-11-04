use crate::*;

/// SchemeInformationBox (`schi`), ISO/IEC 14496-12:2022 Sect 8.12.7.
/// Its contents depend on the scheme; Common Encryption puts a `tenc` here.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Schi {
    pub tenc: Option<Tenc>,
}

impl Atom for Schi {
    const KIND: FourCC = FourCC::new(b"schi");

    nested! {
        required: [],
        optional: [ Tenc ],
        multiple: [],
    }
}
