use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hvc1 {
    pub visual: Visual,
    pub hvcc: Hvcc,
    // TODO support SPS/PPS/VPS atoms
    pub btrt: Option<Btrt>,
    pub colr: Option<Colr>,
    pub pasp: Option<Pasp>,
    pub taic: Option<Taic>,
    pub fiel: Option<Fiel>,
}

impl Atom for Hvc1 {
    const KIND: FourCC = FourCC::new(b"hvc1");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let visual = Visual::decode(buf)?;

        let mut hvcc = None;
        let mut btrt = None;
        let mut colr = None;
        let mut pasp = None;
        let mut taic = None;
        let mut fiel = None;
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Hvcc(atom) => hvcc = atom.into(),
                Any::Btrt(atom) => btrt = atom.into(),
                Any::Colr(atom) => colr = atom.into(),
                Any::Pasp(atom) => pasp = atom.into(),
                Any::Taic(atom) => taic = atom.into(),
                Any::Fiel(atom) => fiel = atom.into(),
                unknown => Self::decode_unknown(&unknown)?,
            }
        }

        Ok(Self {
            visual,
            hvcc: hvcc.ok_or(Error::MissingBox(Hvcc::KIND))?,
            btrt,
            colr,
            pasp,
            taic,
            fiel,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.visual.encode(buf)?;
        self.hvcc.encode(buf)?;
        if self.btrt.is_some() {
            self.btrt.encode(buf)?;
        }
        if self.colr.is_some() {
            self.colr.encode(buf)?;
        }
        if self.pasp.is_some() {
            self.pasp.encode(buf)?;
        }
        if self.taic.is_some() {
            self.taic.encode(buf)?;
        }
        if self.fiel.is_some() {
            self.fiel.encode(buf)?;
        }
        Ok(())
    }
}
