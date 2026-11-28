use crate::*;

/// ProtectionSchemeInfoBox (`sinf`), ISO/IEC 14496-12:2026 Section 13.4.2.
///
/// `schm` and `schi` occur together or not at all. For the Common Encryption schemes
/// `cenc`, `cbc1`, `cens` and `cbcs` (ISO/IEC 23001-7:2023 Section 10.1 to 10.4),
/// decoding and encoding also require a `tenc` in `schi`. Other schemes, `sve1`
/// (Section 10.5) included, are not checked.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sinf {
    pub frma: Frma,
    pub schm: Option<Schm>,
    pub schi: Option<Schi>,
}

impl Atom for Sinf {
    const KIND: FourCC = FourCC::new(b"sinf");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let mut frma = None;
        let mut schm = None;
        let mut schi = None;
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Frma(atom) => frma.insert_once(atom)?,
                Any::Schm(atom) => schm.insert_once(atom)?,
                Any::Schi(atom) => schi.insert_once(atom)?,
                Any::Skip(atom) => tracing::debug!(size = atom.zeroed.size, "skipping skip box"),
                Any::Free(atom) => tracing::debug!(size = atom.zeroed.size, "skipping free box"),
                unknown => Self::decode_unknown(&unknown)?,
            }
        }
        skip_trailing_padding(buf);

        let sinf = Sinf {
            frma: frma.ok_or(Error::MissingBox(Frma::KIND))?,
            schm,
            schi,
        };
        sinf.check()?;
        Ok(sinf)
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.check()?;
        self.frma.encode(buf)?;
        self.schm.encode(buf)?;
        self.schi.encode(buf)
    }
}

impl Sinf {
    fn check(&self) -> Result<()> {
        if self.schm.is_some() != self.schi.is_some() {
            return Err(Error::InvalidCombination(
                "sinf needs both schm and schi, or neither",
            ));
        }
        self.check_common_encryption()
    }

    fn check_common_encryption(&self) -> Result<()> {
        let Some(schm) = &self.schm else {
            return Ok(());
        };
        if !matches!(
            &<[u8; 4]>::from(schm.scheme_type),
            b"cenc" | b"cbc1" | b"cens" | b"cbcs"
        ) {
            return Ok(());
        }
        match self.schi.as_ref().and_then(|schi| schi.tenc.as_ref()) {
            Some(_) => Ok(()),
            None => Err(Error::InvalidCombination(
                "a Common Encryption scheme needs a tenc in schi",
            )),
        }
    }
}
