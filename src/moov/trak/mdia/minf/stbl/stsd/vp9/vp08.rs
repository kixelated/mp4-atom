use crate::*;

// https://www.webmproject.org/vp9/mp4/
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vp08 {
    pub visual: Visual,
    pub vpcc: VpcC,
    pub btrt: Option<Btrt>,
    pub colr: Option<Colr>,
    pub pasp: Option<Pasp>,
    pub sv3d: Option<Sv3d>,
    pub st3d: Option<St3d>,
    pub vexu: Option<Vexu>,
    pub hfov: Option<Hfov>,
}

impl Atom for Vp08 {
    const KIND: FourCC = FourCC::new(b"vp08");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let visual = Visual::decode(buf)?;

        let mut vpcc = None;
        let mut btrt = None;
        let mut colr = None;
        let mut pasp = None;
        let mut sv3d = None;
        let mut st3d = None;
        let mut vexu = None;
        let mut hfov = None;
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::VpcC(atom) => vpcc = atom.into(),
                Any::Btrt(atom) => btrt = atom.into(),
                Any::Colr(atom) => colr = atom.into(),
                Any::Pasp(atom) => pasp = atom.into(),
                Any::Sv3d(atom) => sv3d = atom.into(),
                Any::St3d(atom) => st3d = atom.into(),
                Any::Vexu(atom) => vexu = atom.into(),
                Any::Hfov(atom) => hfov = atom.into(),
                unknown => Self::decode_unknown(&unknown)?,
            }
        }
        skip_trailing_padding(buf);

        Ok(Self {
            visual,
            vpcc: vpcc.ok_or(Error::MissingBox(VpcC::KIND))?,
            btrt,
            colr,
            pasp,
            sv3d,
            st3d,
            vexu,
            hfov,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.visual.encode(buf)?;
        self.vpcc.encode(buf)?;
        self.btrt.encode(buf)?;
        self.colr.encode(buf)?;
        self.pasp.encode(buf)?;
        self.sv3d.encode(buf)?;
        self.st3d.encode(buf)?;
        self.vexu.encode(buf)?;
        self.hfov.encode(buf)?;

        Ok(())
    }
}
