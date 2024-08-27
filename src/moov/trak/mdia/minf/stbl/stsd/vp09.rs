use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Vp09 {
    pub start_code: u16,
    pub data_reference_index: u16,
    pub width: u16,
    pub height: u16,
    pub horizresolution: (u16, u16),
    pub vertresolution: (u16, u16),
    pub frame_count: u16,
    pub compressorname: [u8; 32],
    pub depth: u16,
    pub end_code: u16,
    pub vpcc: Vpcc,
}

impl Vp09 {
    pub const DEFAULT_START_CODE: u16 = 0;
    pub const DEFAULT_END_CODE: u16 = 0xFFFF;
    pub const DEFAULT_DATA_REFERENCE_INDEX: u16 = 1;
    pub const DEFAULT_HORIZRESOLUTION: (u16, u16) = (0x48, 0x00);
    pub const DEFAULT_VERTRESOLUTION: (u16, u16) = (0x48, 0x00);
    pub const DEFAULT_FRAME_COUNT: u16 = 1;
    pub const DEFAULT_COMPRESSORNAME: [u8; 32] = [0; 32];
    pub const DEFAULT_DEPTH: u16 = 24;

    pub fn new(config: &Vp9Config) -> Self {
        Vp09 {
            start_code: Vp09::DEFAULT_START_CODE,
            data_reference_index: Vp09::DEFAULT_DATA_REFERENCE_INDEX,
            width: config.width,
            height: config.height,
            horizresolution: Vp09::DEFAULT_HORIZRESOLUTION,
            vertresolution: Vp09::DEFAULT_VERTRESOLUTION,
            frame_count: Vp09::DEFAULT_FRAME_COUNT,
            compressorname: Vp09::DEFAULT_COMPRESSORNAME,
            depth: Vp09::DEFAULT_DEPTH,
            end_code: Vp09::DEFAULT_END_CODE,
            vpcc: Vpcc {
                profile: 0,
                level: 0x1F,
                bit_depth: Vpcc::DEFAULT_BIT_DEPTH,
                chroma_subsampling: 0,
                video_full_range_flag: false,
                color_primaries: 0,
                transfer_characteristics: 0,
                matrix_coefficients: 0,
                codec_initialization_data_size: 0,
            },
        }
    }
}

impl AtomExt for Vp09 {
    type Ext = ();

    const KIND: FourCC = FourCC::new(b"vp09");

    fn decode_atom(buf: &mut Buf, _ext: ()) -> Result<Self> {
        let start_code: u16 = buf.decode()?;
        let data_reference_index: u16 = buf.decode()?;
        buf.skip(16)?;
        let width: u16 = buf.decode()?;
        let height: u16 = buf.decode()?;
        let horizresolution: (u16, u16) = (buf.decode()?, buf.decode()?);
        let vertresolution: (u16, u16) = (buf.decode()?, buf.decode()?);
        buf.skip(4)?;
        let frame_count: u16 = buf.decode()?;
        let compressorname = buf.decode()?;
        let depth: u16 = buf.decode()?;
        let end_code: u16 = buf.decode()?;

        let vpcc = Vpcc::decode(buf)?;

        Ok(Self {
            start_code,
            data_reference_index,
            width,
            height,
            horizresolution,
            vertresolution,
            frame_count,
            compressorname,
            depth,
            end_code,
            vpcc,
        })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        self.start_code.encode(buf)?;
        self.data_reference_index.encode(buf)?;
        buf.zero(16)?;
        self.width.encode(buf)?;
        self.height.encode(buf)?;
        buf.u16(self.horizresolution.0)?;
        buf.u16(self.horizresolution.1)?;
        buf.u16(self.vertresolution.0)?;
        buf.u16(self.vertresolution.1)?;
        buf.zero(4)?;
        self.frame_count.encode(buf)?;
        self.compressorname.encode(buf)?;
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
        let expected = Vp09::new(&Vp9Config {
            width: 1920,
            height: 1080,
        });
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Vp09::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
