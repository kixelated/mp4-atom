use crate::*;

// https://www.webmproject.org/vp9/mp4/
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vp09 {
    pub visual: Visual,
    pub vpcc: VpcC,
}

impl Atom for Vp09 {
    const KIND: FourCC = FourCC::new(b"vp09");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let visual = Visual::decode(buf)?;

        let mut vpcc = None;
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::VpcC(atom) => vpcc = atom.into(),
                _ => tracing::warn!("unknown atom: {:?}", atom),
            }
        }

        Ok(Self {
            visual,
            vpcc: vpcc.ok_or(Error::MissingBox(VpcC::KIND))?,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.visual.encode(buf)?;
        self.vpcc.encode(buf)?;

        Ok(())
    }
}

ext! {
    name: Vpcc,
    versions: [1],
    flags: {}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VpcC {
    pub profile: u8,
    pub level: u8,
    pub bit_depth: u8,
    pub chroma_subsampling: u8,
    pub video_full_range_flag: bool,
    pub color_primaries: u8,
    pub transfer_characteristics: u8,
    pub matrix_coefficients: u8,
    pub codec_initialization_data: Vec<u8>,
}

impl AtomExt for VpcC {
    const KIND_EXT: FourCC = FourCC::new(b"vpcC");

    type Ext = VpccExt;

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: VpccExt) -> Result<Self> {
        let profile = u8::decode(buf)?;
        let level = u8::decode(buf)?;
        let (bit_depth, chroma_subsampling, video_full_range_flag) = {
            let b = u8::decode(buf)?;
            (b >> 4, (b >> 1) & 0x01, b & 0x01 == 1)
        };
        let color_primaries = u8::decode(buf)?;
        let transfer_characteristics = u8::decode(buf)?;
        let matrix_coefficients = u8::decode(buf)?;
        let _codec_initialization_data_size = u16::decode(buf)?;
        let codec_initialization_data = Vec::decode(buf)?; // assert same as data_size

        Ok(Self {
            profile,
            level,
            bit_depth,
            chroma_subsampling,
            video_full_range_flag,
            color_primaries,
            transfer_characteristics,
            matrix_coefficients,
            codec_initialization_data,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<VpccExt> {
        self.profile.encode(buf)?;
        self.level.encode(buf)?;
        ((self.bit_depth << 4)
            | (self.chroma_subsampling << 1)
            | (self.video_full_range_flag as u8))
            .encode(buf)?;
        self.color_primaries.encode(buf)?;
        self.transfer_characteristics.encode(buf)?;
        self.matrix_coefficients.encode(buf)?;
        (self.codec_initialization_data.len() as u16).encode(buf)?;
        self.codec_initialization_data.encode(buf)?;

        Ok(VpccVersion::V1.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vpcc() {
        let expected = VpcC {
            profile: 0,
            level: 0x1F,
            bit_depth: 8,
            chroma_subsampling: 0,
            video_full_range_flag: false,
            color_primaries: 0,
            transfer_characteristics: 0,
            matrix_coefficients: 0,
            codec_initialization_data: vec![],
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = VpcC::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_vp09() {
        let expected = Vp09 {
            visual: Visual {
                width: 1920,
                height: 1080,
                ..Default::default()
            },
            vpcc: VpcC::default(),
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Vp09::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
