use crate::*;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hvcc {
    pub configuration_version: u8,
    pub general_profile_space: u8,
    pub general_tier_flag: bool,
    pub general_profile_idc: u8,
    pub general_profile_compatibility_flags: [u8; 4],
    pub general_constraint_indicator_flags: [u8; 6],
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HvcCArray {
    pub completeness: bool,
    pub nal_unit_type: u8,
    pub nalus: Vec<Vec<u8>>,
}

impl Atom for Hvcc {
    const KIND: FourCC = FourCC::new(b"hvcC");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let configuration_version = u8::decode(buf)?;
        let params = u8::decode(buf)?;
        let general_profile_space = (params & 0b11000000) >> 6;
        let general_tier_flag = ((params & 0b00100000) >> 5) != 0;
        let general_profile_idc = params & 0b00011111;

        let general_profile_compatibility_flags = <[u8; 4]>::decode(buf)?;
        let general_constraint_indicator_flags = <[u8; 6]>::decode(buf)?;
        let general_level_idc = u8::decode(buf)?;
        let min_spatial_segmentation_idc = u16::decode(buf)? & 0x0FFF;
        let parallelism_type = u8::decode(buf)? & 0b11;
        let chroma_format_idc = u8::decode(buf)? & 0b11;
        let bit_depth_luma_minus8 = u8::decode(buf)? & 0b111;
        let bit_depth_chroma_minus8 = u8::decode(buf)? & 0b111;
        let avg_frame_rate = u16::decode(buf)?;

        let params = u8::decode(buf)?;
        let constant_frame_rate = (params & 0b11000000) >> 6;
        let num_temporal_layers = (params & 0b00111000) >> 3;
        let temporal_id_nested = ((params & 0b00000100) >> 2) != 0;
        let length_size_minus_one = params & 0b000011;

        let num_of_arrays = u8::decode(buf)?;

        let mut arrays = Vec::with_capacity(num_of_arrays as _);
        for _ in 0..num_of_arrays {
            let params = u8::decode(buf)?;
            let num_nalus = u16::decode(buf)?;
            let mut nalus = Vec::with_capacity(num_nalus as usize);

            for _ in 0..num_nalus {
                let size = u16::decode(buf)? as usize;
                let data = Vec::decode_exact(buf, size)?;
                nalus.push(data)
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
            general_constraint_indicator_flags,
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

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.configuration_version.encode(buf)?;
        let general_profile_space = (self.general_profile_space & 0b11) << 6;
        let general_tier_flag = u8::from(self.general_tier_flag) << 5;
        let general_profile_idc = self.general_profile_idc & 0b11111;

        (general_profile_space | general_tier_flag | general_profile_idc).encode(buf)?;
        self.general_profile_compatibility_flags.encode(buf)?;
        self.general_constraint_indicator_flags.encode(buf)?;
        self.general_level_idc.encode(buf)?;

        (0xF000 | (self.min_spatial_segmentation_idc & 0x0FFF)).encode(buf)?;
        (0b11111100 | (self.parallelism_type & 0b11)).encode(buf)?;
        (0b11111100 | (self.chroma_format_idc & 0b11)).encode(buf)?;
        (0b11111000 | (self.bit_depth_luma_minus8 & 0b111)).encode(buf)?;
        (0b11111000 | (self.bit_depth_chroma_minus8 & 0b111)).encode(buf)?;
        self.avg_frame_rate.encode(buf)?;

        let constant_frame_rate = (self.constant_frame_rate & 0b11) << 6;
        let num_temporal_layers = (self.num_temporal_layers & 0b111) << 3;
        let temporal_id_nested = u8::from(self.temporal_id_nested) << 2;
        let length_size_minus_one = self.length_size_minus_one & 0b11;
        (constant_frame_rate | num_temporal_layers | temporal_id_nested | length_size_minus_one)
            .encode(buf)?;
        (self.arrays.len() as u8).encode(buf)?;
        for arr in &self.arrays {
            ((arr.nal_unit_type & 0b111111) | (u8::from(arr.completeness) << 7)).encode(buf)?;
            (arr.nalus.len() as u16).encode(buf)?;

            for nalu in &arr.nalus {
                (nalu.len() as u16).encode(buf)?;
                nalu.encode(buf)?;
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
            btrt: None,
            colr: None,
            pasp: None,
            taic: None,
            fiel: None,
            ccst: None,
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Hev1::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    // From libheif encoded image, essentially x265 underneath
    const ENCODED_HVCC_LIBHEIF: &[u8] = &[
        0x00, 0x00, 0x00, 0x7e, 0x68, 0x76, 0x63, 0x43, 0x01, 0x01, 0x60, 0x00, 0x00, 0x00, 0x90,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x78, 0xf0, 0x00, 0xfc, 0xfd, 0xf8, 0xf8, 0x00, 0x00, 0x0f,
        0x03, 0x20, 0x00, 0x01, 0x00, 0x19, 0x40, 0x01, 0x0c, 0x01, 0xff, 0xff, 0x01, 0x60, 0x00,
        0x00, 0x03, 0x00, 0x90, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x78, 0x99, 0x8a, 0x02,
        0x40, 0x21, 0x00, 0x01, 0x00, 0x30, 0x42, 0x01, 0x01, 0x01, 0x60, 0x00, 0x00, 0x03, 0x00,
        0x90, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x78, 0xa0, 0x02, 0x80, 0x80, 0x35, 0x9f,
        0x59, 0x66, 0x62, 0xa4, 0x91, 0x26, 0xbf, 0xfc, 0x1a, 0xb0, 0x1a, 0xac, 0x04, 0x00, 0x00,
        0x03, 0x00, 0x04, 0x00, 0x00, 0x03, 0x00, 0x64, 0x20, 0x22, 0x00, 0x01, 0x00, 0x07, 0x44,
        0x01, 0xc1, 0x72, 0xb6, 0x62, 0x40,
    ];

    #[test]
    fn test_hvcc_libheif_decode() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_HVCC_LIBHEIF);

        let hvcc = Hvcc {
            configuration_version: 1,
            general_profile_space: 0,
            general_tier_flag: false,
            general_profile_idc: 1,
            general_profile_compatibility_flags: [96, 0, 0, 0],
            general_constraint_indicator_flags: [144, 0, 0, 0, 0, 0],
            general_level_idc: 120,
            min_spatial_segmentation_idc: 0,
            parallelism_type: 0,
            chroma_format_idc: 1,
            bit_depth_luma_minus8: 0,
            bit_depth_chroma_minus8: 0,
            avg_frame_rate: 0,
            constant_frame_rate: 0,
            num_temporal_layers: 1,
            temporal_id_nested: true,
            length_size_minus_one: 3,
            arrays: vec![
                HvcCArray {
                    completeness: false,
                    nal_unit_type: 32,
                    nalus: vec![vec![
                        64, 1, 12, 1, 255, 255, 1, 96, 0, 0, 3, 0, 144, 0, 0, 3, 0, 0, 3, 0, 120,
                        153, 138, 2, 64,
                    ]],
                },
                HvcCArray {
                    completeness: false,
                    nal_unit_type: 33,
                    nalus: vec![vec![
                        66, 1, 1, 1, 96, 0, 0, 3, 0, 144, 0, 0, 3, 0, 0, 3, 0, 120, 160, 2, 128,
                        128, 53, 159, 89, 102, 98, 164, 145, 38, 191, 252, 26, 176, 26, 172, 4, 0,
                        0, 3, 0, 4, 0, 0, 3, 0, 100, 32,
                    ]],
                },
                HvcCArray {
                    completeness: false,
                    nal_unit_type: 34,
                    nalus: vec![vec![68, 1, 193, 114, 182, 98, 64]],
                },
            ],
        };
        let decoded = Hvcc::decode(buf).unwrap();
        assert_eq!(decoded, hvcc);
    }

    #[test]
    fn test_hvcc_libheif_encode() {
        let hvcc = Hvcc {
            configuration_version: 1,
            general_profile_space: 0,
            general_tier_flag: false,
            general_profile_idc: 1,
            general_profile_compatibility_flags: [96, 0, 0, 0],
            general_constraint_indicator_flags: [144, 0, 0, 0, 0, 0],
            general_level_idc: 120,
            min_spatial_segmentation_idc: 0,
            parallelism_type: 0,
            chroma_format_idc: 1,
            bit_depth_luma_minus8: 0,
            bit_depth_chroma_minus8: 0,
            avg_frame_rate: 0,
            constant_frame_rate: 0,
            num_temporal_layers: 1,
            temporal_id_nested: true,
            length_size_minus_one: 3,
            arrays: vec![
                HvcCArray {
                    completeness: false,
                    nal_unit_type: 32,
                    nalus: vec![vec![
                        64, 1, 12, 1, 255, 255, 1, 96, 0, 0, 3, 0, 144, 0, 0, 3, 0, 0, 3, 0, 120,
                        153, 138, 2, 64,
                    ]],
                },
                HvcCArray {
                    completeness: false,
                    nal_unit_type: 33,
                    nalus: vec![vec![
                        66, 1, 1, 1, 96, 0, 0, 3, 0, 144, 0, 0, 3, 0, 0, 3, 0, 120, 160, 2, 128,
                        128, 53, 159, 89, 102, 98, 164, 145, 38, 191, 252, 26, 176, 26, 172, 4, 0,
                        0, 3, 0, 4, 0, 0, 3, 0, 100, 32,
                    ]],
                },
                HvcCArray {
                    completeness: false,
                    nal_unit_type: 34,
                    nalus: vec![vec![68, 1, 193, 114, 182, 98, 64]],
                },
            ],
        };
        let mut buf = Vec::new();
        hvcc.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_HVCC_LIBHEIF);
    }

    // From MPEG file format tests - published/nalu/hevc/hvc1_only.mp4
    // probably gpac over x265
    const ENCODED_HVCC_MPEG: &[u8] = &[
        0x00, 0x00, 0x00, 0xa8, 0x68, 0x76, 0x63, 0x43, 0x01, 0x01, 0x60, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x7b, 0xf0, 0x00, 0xfd, 0xfd, 0xf8, 0xf8, 0x00, 0x00, 0x0f,
        0x03, 0xa0, 0x00, 0x01, 0x00, 0x41, 0x40, 0x01, 0x0c, 0x11, 0xff, 0xff, 0x01, 0x60, 0x00,
        0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x7b, 0x94, 0x90,
        0x57, 0x00, 0x00, 0x03, 0x03, 0xe9, 0x00, 0x00, 0xea, 0x60, 0x7f, 0x7b, 0x10, 0x00, 0x04,
        0x30, 0x24, 0xcf, 0x75, 0x88, 0x0f, 0x00, 0x08, 0x82, 0x80, 0x7b, 0x07, 0x80, 0x04, 0x38,
        0xa0, 0x0e, 0x0a, 0x52, 0x7b, 0x90, 0xa0, 0x20, 0x20, 0x2c, 0x10, 0xa1, 0x00, 0x01, 0x00,
        0x31, 0x42, 0x01, 0x01, 0x01, 0x60, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x03, 0x00, 0x7b, 0xa0, 0x07, 0x82, 0x00, 0x88, 0x7d, 0xe5, 0x94, 0x99, 0x24,
        0x6d, 0x86, 0x96, 0x22, 0xaa, 0x4c, 0x4c, 0x32, 0xfb, 0x3e, 0xbc, 0xdf, 0x96, 0x7d, 0x78,
        0x51, 0x18, 0x9c, 0xbb, 0x64, 0xa2, 0x00, 0x01, 0x00, 0x08, 0x44, 0x01, 0xc1, 0xa5, 0x58,
        0x11, 0xd0, 0x2a,
    ];

    #[test]
    fn test_hvcc_mpeg_decode() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_HVCC_MPEG);

        /* From MPEG file dump (essentially GPAC):
                      "HEVCDecoderConfigurationRecord": {
                      "@nal_unit_size": "4",
                      "@configurationVersion": "1",
                      "@profile_space": "0",
                      "@tier_flag": "0",
                      "@profile_idc": "1",
                      "@general_profile_compatibility_flags": "60000000",
                      "@progressive_source_flag": "0",
                      "@interlaced_source_flag": "0",
                      "@non_packed_constraint_flag": "0",
                      "@frame_only_constraint_flag": "0",
                      "@constraint_indicator_flags": "0",
                      "@level_idc": "123",
                      "@min_spatial_segmentation_idc": "0",
                      "@parallelismType": "1",
                      "@chroma_format": "YUV 4:2:0",
                      "@luma_bit_depth": "8",
                      "@chroma_bit_depth": "8",
                      "@avgFrameRate": "0",
                      "@constantFrameRate": "0",
                      "@numTemporalLayers": "1",
                      "@temporalIdNested": "1",
                      "ParameterSetArray": [
                        {
                          "@nalu_type": "32",
                          "@complete_set": "1",
                          "ParameterSet": {
                            "@size": "65",
                            "@content": "data:application/octet-string,40010C11FFFF0160000003000003000003000003007B94905700000303E90000EA607F7B1000043024CF75880F000882807B07800438A00E0A527B90A020202C10"
                          }
                        },
                        {
                          "@nalu_type": "33",
                          "@complete_set": "1",
                          "ParameterSet": {
                            "@size": "49",
                            "@content": "data:application/octet-string,4201010160000003000003000003000003007BA0078200887DE59499246D869622AA4C4C32FB3EBCDF967D7851189CBB64"
                          }
                        },
                        {
                          "@nalu_type": "34",
                          "@complete_set": "1",
                          "ParameterSet": {
                            "@size": "8",
                            "@content": "data:application/octet-string,4401C1A55811D02A"
                          }
                        }
                      ]
                    }
                  },
        */
        let hvcc = Hvcc {
            configuration_version: 1,
            general_profile_space: 0,
            general_tier_flag: false,
            general_profile_idc: 1,
            general_profile_compatibility_flags: [0x60, 0x00, 0x00, 0x00],
            general_constraint_indicator_flags: [0, 0, 0, 0, 0, 0],
            general_level_idc: 123,
            min_spatial_segmentation_idc: 0,
            parallelism_type: 1,
            chroma_format_idc: 1,
            bit_depth_luma_minus8: 0,
            bit_depth_chroma_minus8: 0,
            avg_frame_rate: 0,
            constant_frame_rate: 0,
            num_temporal_layers: 1,
            temporal_id_nested: true,
            length_size_minus_one: 3,
            arrays: vec![
                HvcCArray {
                    completeness: true,
                    nal_unit_type: 32,
                    nalus: vec![vec![
                        0x40, 0x01, 0x0C, 0x11, 0xFF, 0xFF, 0x01, 0x60, 0x00, 0x00, 0x03, 0x00,
                        0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x7B, 0x94, 0x90,
                        0x57, 0x00, 0x00, 0x03, 0x03, 0xE9, 0x00, 0x00, 0xEA, 0x60, 0x7F, 0x7B,
                        0x10, 0x00, 0x04, 0x30, 0x24, 0xCF, 0x75, 0x88, 0x0F, 0x00, 0x08, 0x82,
                        0x80, 0x7B, 0x07, 0x80, 0x04, 0x38, 0xA0, 0x0E, 0x0A, 0x52, 0x7B, 0x90,
                        0xA0, 0x20, 0x20, 0x2C, 0x10,
                    ]],
                },
                HvcCArray {
                    completeness: true,
                    nal_unit_type: 33,
                    nalus: vec![vec![
                        0x42, 0x01, 0x01, 0x01, 0x60, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00,
                        0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x7B, 0xA0, 0x07, 0x82, 0x00, 0x88,
                        0x7D, 0xE5, 0x94, 0x99, 0x24, 0x6D, 0x86, 0x96, 0x22, 0xAA, 0x4C, 0x4C,
                        0x32, 0xFB, 0x3E, 0xBC, 0xDF, 0x96, 0x7D, 0x78, 0x51, 0x18, 0x9C, 0xBB,
                        0x64,
                    ]],
                },
                HvcCArray {
                    completeness: true,
                    nal_unit_type: 34,
                    nalus: vec![vec![0x44, 0x01, 0xC1, 0xA5, 0x58, 0x11, 0xD0, 0x2A]],
                },
            ],
        };
        let decoded = Hvcc::decode(buf).unwrap();
        assert_eq!(decoded, hvcc);
    }

    #[test]
    fn test_hvcc_mpeg_encode() {
        let hvcc = Hvcc {
            configuration_version: 1,
            general_profile_space: 0,
            general_tier_flag: false,
            general_profile_idc: 1,
            general_profile_compatibility_flags: [0x60, 0x00, 0x00, 0x00],
            general_constraint_indicator_flags: [0, 0, 0, 0, 0, 0],
            general_level_idc: 123,
            min_spatial_segmentation_idc: 0,
            parallelism_type: 1,
            chroma_format_idc: 1,
            bit_depth_luma_minus8: 0,
            bit_depth_chroma_minus8: 0,
            avg_frame_rate: 0,
            constant_frame_rate: 0,
            num_temporal_layers: 1,
            temporal_id_nested: true,
            length_size_minus_one: 3,
            arrays: vec![
                HvcCArray {
                    completeness: true,
                    nal_unit_type: 32,
                    nalus: vec![vec![
                        0x40, 0x01, 0x0C, 0x11, 0xFF, 0xFF, 0x01, 0x60, 0x00, 0x00, 0x03, 0x00,
                        0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x7B, 0x94, 0x90,
                        0x57, 0x00, 0x00, 0x03, 0x03, 0xE9, 0x00, 0x00, 0xEA, 0x60, 0x7F, 0x7B,
                        0x10, 0x00, 0x04, 0x30, 0x24, 0xCF, 0x75, 0x88, 0x0F, 0x00, 0x08, 0x82,
                        0x80, 0x7B, 0x07, 0x80, 0x04, 0x38, 0xA0, 0x0E, 0x0A, 0x52, 0x7B, 0x90,
                        0xA0, 0x20, 0x20, 0x2C, 0x10,
                    ]],
                },
                HvcCArray {
                    completeness: true,
                    nal_unit_type: 33,
                    nalus: vec![vec![
                        0x42, 0x01, 0x01, 0x01, 0x60, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00,
                        0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x7B, 0xA0, 0x07, 0x82, 0x00, 0x88,
                        0x7D, 0xE5, 0x94, 0x99, 0x24, 0x6D, 0x86, 0x96, 0x22, 0xAA, 0x4C, 0x4C,
                        0x32, 0xFB, 0x3E, 0xBC, 0xDF, 0x96, 0x7D, 0x78, 0x51, 0x18, 0x9C, 0xBB,
                        0x64,
                    ]],
                },
                HvcCArray {
                    completeness: true,
                    nal_unit_type: 34,
                    nalus: vec![vec![0x44, 0x01, 0xC1, 0xA5, 0x58, 0x11, 0xD0, 0x2A]],
                },
            ],
        };
        let mut buf = Vec::new();
        hvcc.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_HVCC_MPEG);
    }
}
