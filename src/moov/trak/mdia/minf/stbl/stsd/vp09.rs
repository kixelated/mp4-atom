use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vp09 {
    pub start_code: u16,
    pub data_reference_index: u16,
    pub width: u16,
    pub height: u16,
    pub horizresolution: FixedPoint<u16>,
    pub vertresolution: FixedPoint<u16>,
    pub frame_count: u16,
    pub compressor: Compressor,
    pub depth: u16,
    pub end_code: u16,
    pub vpcc: Vpcc,
}

impl Default for Vp09 {
    fn default() -> Self {
        Vp09 {
            start_code: 0,
            data_reference_index: 1,
            width: 0,
            height: 0,
            horizresolution: 0x48.into(),
            vertresolution: 0x48.into(),
            frame_count: 1,
            compressor: Compressor::default(),
            depth: 24,
            end_code: 0xFFFF,
            vpcc: Vpcc::default(),
        }
    }
}

impl AtomExt for Vp09 {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"vp09");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let start_code = u16::decode(buf)?;
        let data_reference_index = u16::decode(buf)?;
        <[u8; 16]>::decode(buf)?;
        let width = u16::decode(buf)?;
        let height = u16::decode(buf)?;
        let horizresolution = FixedPoint::decode(buf)?;
        let vertresolution = FixedPoint::decode(buf)?;
        <[u8; 4]>::decode(buf)?;
        let frame_count = u16::decode(buf)?;

        let compressor = Compressor::decode(buf)?;
        let depth = u16::decode(buf)?;
        let end_code = u16::decode(buf)?;

        let vpcc = Vpcc::decode(buf)?;

        Ok(Self {
            start_code,
            data_reference_index,
            width,
            height,
            horizresolution,
            vertresolution,
            frame_count,
            compressor,
            depth,
            end_code,
            vpcc,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.start_code.encode(buf)?;
        self.data_reference_index.encode(buf)?;
        [0u8; 16].encode(buf)?;
        self.width.encode(buf)?;
        self.height.encode(buf)?;
        self.horizresolution.encode(buf)?;
        self.vertresolution.encode(buf)?;
        0u32.encode(buf)?;
        self.frame_count.encode(buf)?;
        self.compressor.encode(buf)?;
        self.depth.encode(buf)?;
        self.end_code.encode(buf)?;

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
            width: 1920,
            height: 1080,
            ..Default::default()
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Vp09::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
