use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vp09 {
    pub visual: Visual,
    pub vpcc: Vpcc,
}

impl AtomExt for Vp09 {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"vp09");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let visual = Visual::decode(buf)?;
        let vpcc = Vpcc::decode(buf)?;

        Ok(Self { visual, vpcc })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.visual.encode(buf)?;
        self.vpcc.encode(buf)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vpcc() {
        let expected = Vp09 {
            visual: Visual {
                width: 1920,
                height: 1080,
                ..Default::default()
            },
            vpcc: Vpcc::default(),
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Vp09::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
