use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pasp {
    pub h_spacing: u32,
    pub v_spacing: u32,
}

impl Pasp {
    pub fn new(h_spacing: u32, v_spacing: u32) -> Result<Self> {
        Ok(Self {
            h_spacing,
            v_spacing,
        })
    }
}

impl Atom for Pasp {
    const KIND: FourCC = FourCC::new(b"pasp");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let h_spacing = u32::decode(buf)?;
        let v_spacing = u32::decode(buf)?;

        Ok(Pasp {
            h_spacing,
            v_spacing,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.h_spacing.encode(buf)?;
        self.v_spacing.encode(buf)?;
        Ok(())
    }
}
