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
    const KIND: FourCC = FourCC::new(b"vpcc");

    type Ext = VpccExt;

    fn decode_atom(buf: &mut Buf, ext: VpccExt) -> Result<Self> {
        let profile: u8 = buf.u8()?;
        let level: u8 = buf.u8()?;
        let (bit_depth, chroma_subsampling, video_full_range_flag) = {
            let b = buf.u8()?;
            (b >> 4, b << 4 >> 5, b & 0x01 == 1)
        };
        let transfer_characteristics: u8 = buf.u8()?;
        let matrix_coefficients: u8 = buf.u8()?;
        let codec_initialization_data_size: u16 = buf.decode()?;

        Ok(Self {
            profile,
            level,
            bit_depth,
            chroma_subsampling,
            video_full_range_flag,
            color_primaries: 0,
            transfer_characteristics,
            matrix_coefficients,
            codec_initialization_data_size,
        })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<VpccExt> {
        buf.u8(self.profile)?;
        buf.u8(self.level)?;
        buf.u8((self.bit_depth << 4)
            | (self.chroma_subsampling << 1)
            | (self.video_full_range_flag as u8))?;
        buf.u8(self.color_primaries)?;
        buf.u8(self.transfer_characteristics)?;
        buf.u8(self.matrix_coefficients)?;
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
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Vpcc::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
