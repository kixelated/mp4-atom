use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hev1 {
    pub data_reference_index: u16,
    pub width: u16,
    pub height: u16,
    pub horizresolution: Ratio<u16>,
    pub vertresolution: Ratio<u16>,
    pub frame_count: u16,
    pub depth: u16,
    pub hvcc: Hvcc,
}

impl Default for Hev1 {
    fn default() -> Self {
        Hev1 {
            data_reference_index: 0,
            width: 0,
            height: 0,
            horizresolution: 0x48.into(),
            vertresolution: 0x48.into(),
            frame_count: 1,
            depth: 0x0018,
            hvcc: Hvcc::default(),
        }
    }
}

impl Hev1 {
    pub fn new(config: &HevcConfig) -> Self {
        Hev1 {
            data_reference_index: 1,
            width: config.width,
            height: config.height,
            horizresolution: 0x48.into(),
            vertresolution: 0x48.into(),
            frame_count: 1,
            depth: 0x0018,
            hvcc: Hvcc::new(),
        }
    }
}

impl Atom for Hev1 {
    const KIND: FourCC = FourCC::new(b"hev1");

    fn decode_atom(buf: &mut Buf) -> Result<Self> {
        u32::decode(buf)?; // reserved
        u16::decode(buf)?; // reserved
        let data_reference_index = buf.decode()?;

        u32::decode(buf)?; // pre-defined, reserved
        u64::decode(buf)?; // pre-defined
        u32::decode(buf)?; // pre-defined
        let width = buf.decode()?;
        let height = buf.decode()?;
        let horizresolution = Ratio::<u16>::decode(buf)?;
        let vertresolution = Ratio::<u16>::decode(buf)?;
        u32::decode(buf)?; // reserved
        let frame_count = buf.decode()?;
        buf.skip(4)?; // compressorname
        let depth = buf.decode()?;
        i16::decode(buf)?; // pre-defined

        let hvcc = Hvcc::decode(buf)?;

        Ok(Hev1 {
            data_reference_index,
            width,
            height,
            horizresolution,
            vertresolution,
            frame_count,
            depth,
            hvcc,
        })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        buf.u32(0)?; // reserved
        buf.u16(0)?; // reserved
        self.data_reference_index.encode(buf)?;

        buf.u32(0)?; // pre-defined, reserved
        buf.u64(0)?; // pre-defined
        buf.u32(0)?; // pre-defined
        self.width.encode(buf)?;
        self.height.encode(buf)?;
        self.horizresolution.encode(buf)?;
        self.vertresolution.encode(buf)?;
        buf.u32(0)?; // reserved
        self.frame_count.encode(buf)?;
        // skip compressorname
        buf.skip(4)?;
        self.depth.encode(buf)?;
        buf.i16(-1)?; // pre-defined

        self.hvcc.encode(buf)?;

        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Hvcc {
    pub configuration_version: u8,
    pub general_profile_space: u8,
    pub general_tier_flag: bool,
    pub general_profile_idc: u8,
    pub general_profile_compatibility_flags: u32,
    pub general_constraint_indicator_flag: u64,
    pub general_level_idc: u8,
    pub min_spatial_segmentation_idc: u16,
    pub parallelism_type: u8,
    pub chroma_format_idc: u8,
    pub bit_depth_luma_minus8: u8,
    pub bit_depth_chroma_minus8: u8,
    pub avg_frame_rate: u16,
    pub constant_frame_rate: u8,
    pub num_temporal_layers: u8,
    pub temporal_id_nested: bool,
    pub length_size_minus_one: u8,
    pub arrays: Vec<HvcCArray>,
}

impl Hvcc {
    pub fn new() -> Self {
        Self {
            configuration_version: 1,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HvcCArrayNalu {
    pub size: u16,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HvcCArray {
    pub completeness: bool,
    pub nal_unit_type: u8,
    pub nalus: Vec<HvcCArrayNalu>,
}

impl Atom for Hvcc {
    const KIND: FourCC = FourCC::new(b"hvcC");

    fn decode_atom(buf: &mut Buf) -> Result<Self> {
        let configuration_version = buf.u8()?;
        let params = buf.u8()?;
        let general_profile_space = params & 0b11000000 >> 6;
        let general_tier_flag = (params & 0b00100000 >> 5) > 0;
        let general_profile_idc = params & 0b00011111;

        let general_profile_compatibility_flags = u32::decode(buf)?;
        let general_constraint_indicator_flag = buf.u48()?;
        let general_level_idc = buf.u8()?;
        let min_spatial_segmentation_idc = buf.decode()? & 0x0FFF;
        let parallelism_type = buf.u8()? & 0b11;
        let chroma_format_idc = buf.u8()? & 0b11;
        let bit_depth_luma_minus8 = buf.u8()? & 0b111;
        let bit_depth_chroma_minus8 = buf.u8()? & 0b111;
        let avg_frame_rate = buf.decode()?;

        let params = buf.u8()?;
        let constant_frame_rate = params & 0b11000000 >> 6;
        let num_temporal_layers = params & 0b00111000 >> 3;
        let temporal_id_nested = (params & 0b00000100 >> 2) > 0;
        let length_size_minus_one = params & 0b000011;

        let num_of_arrays = buf.u8()?;

        let mut arrays = Vec::with_capacity(num_of_arrays as _);
        for _ in 0..num_of_arrays {
            let params = buf.u8()?;
            let num_nalus = buf.decode()?;
            let mut nalus = Vec::with_capacity(num_nalus as usize);

            for _ in 0..num_nalus {
                let size = buf.u16()?;
                let data = buf.bytes(size as usize)?;
                nalus.push(HvcCArrayNalu { size, data })
            }

            arrays.push(HvcCArray {
                completeness: (params & 0b10000000) > 0,
                nal_unit_type: params & 0b111111,
                nalus,
            });
        }

        Ok(Hvcc {
            configuration_version,
            general_profile_space,
            general_tier_flag,
            general_profile_idc,
            general_profile_compatibility_flags,
            general_constraint_indicator_flag,
            general_level_idc,
            min_spatial_segmentation_idc,
            parallelism_type,
            chroma_format_idc,
            bit_depth_luma_minus8,
            bit_depth_chroma_minus8,
            avg_frame_rate,
            constant_frame_rate,
            num_temporal_layers,
            temporal_id_nested,
            length_size_minus_one,
            arrays,
        })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        buf.u8(self.configuration_version)?;
        let general_profile_space = (self.general_profile_space & 0b11) << 6;
        let general_tier_flag = u8::from(self.general_tier_flag) << 5;
        let general_profile_idc = self.general_profile_idc & 0b11111;

        buf.u8(general_profile_space | general_tier_flag | general_profile_idc)?;
        self.general_profile_compatibility_flags.encode(buf)?;
        self.general_constraint_indicator_flag.encode(buf)?;
        buf.u8(self.general_level_idc)?;

        buf.u16(self.min_spatial_segmentation_idc & 0x0FFF)?;
        buf.u8(self.parallelism_type & 0b11)?;
        buf.u8(self.chroma_format_idc & 0b11)?;
        buf.u8(self.bit_depth_luma_minus8 & 0b111)?;
        buf.u8(self.bit_depth_chroma_minus8 & 0b111)?;
        self.avg_frame_rate.encode(buf)?;

        let constant_frame_rate = (self.constant_frame_rate & 0b11) << 6;
        let num_temporal_layers = (self.num_temporal_layers & 0b111) << 3;
        let temporal_id_nested = u8::from(self.temporal_id_nested) << 2;
        let length_size_minus_one = self.length_size_minus_one & 0b11;
        buf.u8(constant_frame_rate
            | num_temporal_layers
            | temporal_id_nested
            | length_size_minus_one)?;
        buf.u8(self.arrays.len() as u8)?;
        for arr in &self.arrays {
            buf.u8((arr.nal_unit_type & 0b111111) | u8::from(arr.completeness) << 7)?;
            buf.u16(arr.nalus.len() as _)?;

            for nalu in &arr.nalus {
                nalu.size.encode(buf)?;
                buf.bytes(&nalu.data)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hev1() {
        let expected = Hev1 {
            data_reference_index: 1,
            width: 320,
            height: 240,
            horizresolution: 0x48.into(),
            vertresolution: 0x48.into(),
            frame_count: 1,
            depth: 24,
            hvcc: Hvcc {
                configuration_version: 1,
                ..Default::default()
            },
        };
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Hev1::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
