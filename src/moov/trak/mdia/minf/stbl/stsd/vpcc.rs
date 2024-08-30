use crate::*;

ext! {
    name: Vpcc,
    versions: [1],
    flags: {}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Vpcc {
    pub profile: u8,
    pub level: u8,
    pub bit_depth: u8,
    pub chroma_subsampling: u8,
    pub video_full_range_flag: bool,
    pub color_primaries: u8,
    pub transfer_characteristics: u8,
    pub matrix_coefficients: u8,
    pub codec_initialization_data_size: u16,
}

impl AtomExt for Vpcc {
    const KIND_EXT: FourCC = FourCC::new(b"vpcc");

    type Ext = VpccExt;

    fn decode_atom_ext(buf: &mut Bytes, _ext: VpccExt) -> Result<Self> {
        let profile = buf.decode()?;
        let level = buf.decode()?;
        let (bit_depth, chroma_subsampling, video_full_range_flag) = {
            let b = u8::decode(buf)?;
            (b >> 4, (b >> 1) & 0x01, b & 0x01 == 1)
        };
        let color_primaries = buf.decode()?;
        let transfer_characteristics = buf.decode()?;
        let matrix_coefficients = buf.decode()?;
        let codec_initialization_data_size = buf.decode()?;

        Ok(Self {
            profile,
            level,
            bit_depth,
            chroma_subsampling,
            video_full_range_flag,
            color_primaries,
            transfer_characteristics,
            matrix_coefficients,
            codec_initialization_data_size,
        })
    }

    fn encode_atom_ext(&self, buf: &mut BytesMut) -> Result<VpccExt> {
        self.profile.encode(buf)?;
        self.level.encode(buf)?;
        ((self.bit_depth << 4)
            | (self.chroma_subsampling << 1)
            | (self.video_full_range_flag as u8))
            .encode(buf)?;
        self.color_primaries.encode(buf)?;
        self.transfer_characteristics.encode(buf)?;
        self.matrix_coefficients.encode(buf)?;
        self.codec_initialization_data_size.encode(buf)?;

        Ok(VpccVersion::V1.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vpcc() {
        let expected = Vpcc {
            profile: 0,
            level: 0x1F,
            bit_depth: 8,
            chroma_subsampling: 0,
            video_full_range_flag: false,
            color_primaries: 0,
            transfer_characteristics: 0,
            matrix_coefficients: 0,
            codec_initialization_data_size: 0,
        };
        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let decoded = Vpcc::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
