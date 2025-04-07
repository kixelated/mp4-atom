use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hvc1 {
    pub visual: Visual,
    pub hvcc: Hvcc,
    // TODO support SPS/PPS/VPS atoms
    pub colr: Option<Colr>,
    pub pasp: Option<Pasp>,
}

impl Atom for Hvc1 {
    const KIND: FourCC = FourCC::new(b"hvc1");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let visual = Visual::decode(buf)?;

        let mut hvcc = None;
        let mut colr = None;
        let mut pasp = None;
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Hvcc(atom) => hvcc = atom.into(),
                Any::Colr(atom) => colr = atom.into(),
                Any::Pasp(atom) => pasp = atom.into(),
                _ => tracing::warn!("unknown atom: {:?}", atom),
            }
        }

        Ok(Self {
            visual,
            hvcc: hvcc.ok_or(Error::MissingBox(Hvcc::KIND))?,
            colr,
            pasp,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.visual.encode(buf)?;
        self.hvcc.encode(buf)?;
        if self.colr.is_some() {
            self.colr.encode(buf)?;
        }
        if self.pasp.is_some() {
            self.pasp.encode(buf)?;
        }

        Ok(())
    }
}
