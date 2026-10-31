use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hev1 {
    pub visual: Visual,
    pub hvcc: Hvcc,
    pub lhvc: Option<Lhvc>,
    pub btrt: Option<Btrt>,
    pub colr: Option<Colr>,
    pub pasp: Option<Pasp>,
    pub taic: Option<Taic>,
    pub fiel: Option<Fiel>,
    pub spatial: Option<Box<Spatial>>,
}

impl Atom for Hev1 {
    const KIND: FourCC = FourCC::new(b"hev1");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let visual = Visual::decode(buf)?;

        let mut hvcc = None;
        let mut lhvc = None;
        let mut btrt = None;
        let mut colr = None;
        let mut pasp = None;
        let mut taic = None;
        let mut fiel = None;
        let mut spatial = None;
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Hvcc(atom) => hvcc.insert_if_none(atom)?,
                Any::Lhvc(atom) => lhvc.insert_if_none(atom)?,
                Any::Btrt(atom) => btrt.insert_if_none(atom)?,
                Any::Colr(atom) => colr.insert_if_none(atom)?,
                Any::Pasp(atom) => pasp.insert_if_none(atom)?,
                Any::Taic(atom) => taic.insert_if_none(atom)?,
                Any::Fiel(atom) => fiel.insert_if_none(atom)?,
                atom => Spatial::decode_child::<Self>(&mut spatial, atom)?,
            }
        }
        skip_trailing_padding(buf);

        Ok(Hev1 {
            visual,
            hvcc: hvcc.ok_or(Error::MissingBox(Hvcc::KIND))?,
            lhvc,
            btrt,
            colr,
            pasp,
            taic,
            fiel,
            spatial,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.visual.encode(buf)?;
        self.hvcc.encode(buf)?;
        self.lhvc.encode(buf)?;
        self.btrt.encode(buf)?;
        self.colr.encode(buf)?;
        if let Some(spatial) = &self.spatial {
            spatial.encode(buf)?;
        }
        self.pasp.encode(buf)?;
        self.taic.encode(buf)?;
        self.fiel.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hev1() {
        let expected = Hev1 {
            visual: Visual {
                data_reference_index: 1,
                width: 320,
                height: 240,
                horizresolution: 0x48.into(),
                vertresolution: 0x48.into(),
                frame_count: 1,
                compressor: "ya boy".into(),
                depth: 24,
            },
            hvcc: Hvcc {
                configuration_version: 1,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Hev1::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
