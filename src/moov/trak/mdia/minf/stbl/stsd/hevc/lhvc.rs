use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lhvc {
    pub configuration_version: u8,
    pub min_spatial_segmentation_idc: u16,
    pub parallelism_type: u8,
    pub num_temporal_layers: u8,
    pub temporal_id_nested: bool,
    pub length_size_minus_one: u8,
    pub arrays: Vec<HvcCArray>,
}

impl Atom for Lhvc {
    const KIND: FourCC = FourCC::new(b"lhvC");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let configuration_version = u8::decode(buf)?;
        let min_spatial_segmentation_idc = u16::decode(buf)? & 0x0FFF;
        let parallelism_type = u8::decode(buf)? & 0b11;
        let temp = u8::decode(buf)?;
        let length_size_minus_one = temp & 0b11;
        let temporal_id_nested = (temp & 0b0000_0100) != 0;
        let num_temporal_layers = (temp & 0b0011_1000) >> 3;
        let num_of_arrays = u8::decode(buf)?;

        let mut arrays = Vec::with_capacity(num_of_arrays.min(8) as _);
        for _ in 0..num_of_arrays {
            let params = u8::decode(buf)?;
            let num_nalus = u16::decode(buf)?;
            let mut nalus = Vec::with_capacity(num_nalus.min(8) as usize);

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

        Ok(Self {
            configuration_version,
            min_spatial_segmentation_idc,
            parallelism_type,
            num_temporal_layers,
            temporal_id_nested,
            length_size_minus_one,
            arrays,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        if self.min_spatial_segmentation_idc > 0x0FFF {
            return Err(Error::InvalidSize);
        }
        if self.parallelism_type > 0b11 {
            return Err(Error::InvalidSize);
        }
        if self.num_temporal_layers > 0b111 {
            return Err(Error::InvalidSize);
        }
        if self.length_size_minus_one > 0b11 {
            return Err(Error::InvalidSize);
        }
        if self.arrays.len() > 0xFF {
            return Err(Error::InvalidSize);
        }
        self.configuration_version.encode(buf)?;
        ((0b1111 << 12) | self.min_spatial_segmentation_idc).encode(buf)?;
        ((0b111111 << 2) | self.parallelism_type).encode(buf)?;
        ((0b11 << 6)
            | (self.num_temporal_layers << 3)
            | (if self.temporal_id_nested { 1 << 2 } else { 0 })
            | self.length_size_minus_one)
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
    use std::vec;

    use super::*;

    // From MPEG File Format Conformance, nalu/l-hevc/lhevc_av1_lhv1.mp4
    const ENCODED_LHVC: &[u8] = &[
        0x00, 0x00, 0x00, 0xa0, 0x6c, 0x68, 0x76, 0x43, 0x01, 0xf0, 0x00, 0xfd, 0xc3, 0x03, 0xa0,
        0x00, 0x01, 0x00, 0x5d, 0x40, 0x01, 0x04, 0x11, 0xff, 0xff, 0x01, 0x80, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x78, 0x94, 0x90, 0x57, 0x00,
        0x00, 0x03, 0x03, 0xe9, 0x00, 0x00, 0xea, 0x60, 0x7f, 0x10, 0x00, 0x04, 0x30, 0x28, 0xe0,
        0x30, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x0b,
        0xb0, 0x78, 0x00, 0x00, 0x03, 0x00, 0x00, 0xf8, 0x80, 0x00, 0x00, 0x03, 0x00, 0x07, 0x8a,
        0xc4, 0x07, 0x80, 0x04, 0x41, 0x40, 0x3d, 0x83, 0xc0, 0x02, 0x1c, 0x50, 0x0f, 0xd8, 0x52,
        0x73, 0x08, 0x50, 0x10, 0x10, 0x16, 0x08, 0xa1, 0x00, 0x01, 0x00, 0x1b, 0x42, 0x09, 0x0e,
        0x85, 0x92, 0x46, 0xd8, 0x69, 0x62, 0x2a, 0xa4, 0xc4, 0xc3, 0x2f, 0xb3, 0xeb, 0xcd, 0xf9,
        0x67, 0xd7, 0x85, 0x11, 0x89, 0xcb, 0xb6, 0xa0, 0x20, 0xa2, 0x00, 0x01, 0x00, 0x0b, 0x44,
        0x09, 0x48, 0x1a, 0x55, 0x81, 0x3d, 0x02, 0x40, 0x3c, 0x84,
    ];

    #[test]
    fn test_lhvc() {
        let mut buf = std::io::Cursor::new(ENCODED_LHVC);

        let lhvc = Lhvc::decode(&mut buf).expect("failed to decode lhvC");

        assert_eq!(
            lhvc,
            Lhvc {
                configuration_version: 1,
                min_spatial_segmentation_idc: 0,
                parallelism_type: 1,
                num_temporal_layers: 0,
                temporal_id_nested: false,
                length_size_minus_one: 3,
                arrays: vec![
                    HvcCArray {
                        completeness: true,
                        nal_unit_type: 32,
                        nalus: vec![vec![
                            0x40, 0x01, 0x04, 0x11, 0xff, 0xff, 0x01, 0x80, 0x00, 0x00, 0x03, 0x00,
                            0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x78, 0x94, 0x90,
                            0x57, 0x00, 0x00, 0x03, 0x03, 0xe9, 0x00, 0x00, 0xea, 0x60, 0x7f, 0x10,
                            0x00, 0x04, 0x30, 0x28, 0xe0, 0x30, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
                            0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x0b, 0xb0, 0x78, 0x00, 0x00,
                            0x03, 0x00, 0x00, 0xf8, 0x80, 0x00, 0x00, 0x03, 0x00, 0x07, 0x8a, 0xc4,
                            0x07, 0x80, 0x04, 0x41, 0x40, 0x3d, 0x83, 0xc0, 0x02, 0x1c, 0x50, 0x0f,
                            0xd8, 0x52, 0x73, 0x08, 0x50, 0x10, 0x10, 0x16, 0x08
                        ]]
                    },
                    HvcCArray {
                        completeness: true,
                        nal_unit_type: 33,
                        nalus: vec![vec![
                            0x42, 0x09, 0x0e, 0x85, 0x92, 0x46, 0xd8, 0x69, 0x62, 0x2a, 0xa4, 0xc4,
                            0xc3, 0x2f, 0xb3, 0xeb, 0xcd, 0xf9, 0x67, 0xd7, 0x85, 0x11, 0x89, 0xcb,
                            0xb6, 0xa0, 0x20
                        ]]
                    },
                    HvcCArray {
                        completeness: true,
                        nal_unit_type: 34,
                        nalus: vec![vec![
                            0x44, 0x09, 0x48, 0x1a, 0x55, 0x81, 0x3d, 0x02, 0x40, 0x3c, 0x84
                        ]]
                    }
                ],
            }
        );

        let mut encoded = Vec::new();
        lhvc.encode(&mut encoded).expect("failed to encode lhvC");
        assert_eq!(encoded, ENCODED_LHVC);
    }

    #[test]
    fn test_lhvc_round_trip_1() {
        let lhvc = Lhvc {
            configuration_version: 1,
            min_spatial_segmentation_idc: 14,
            parallelism_type: 1,
            num_temporal_layers: 1,
            temporal_id_nested: false,
            length_size_minus_one: 0,
            arrays: vec![],
        };

        let mut encoded = Vec::new();
        lhvc.encode(&mut encoded).expect("failed to encode lhvC");
        let mut buf = encoded.as_ref();
        let decoded = Lhvc::decode(&mut buf).unwrap();
        assert_eq!(decoded, lhvc);
    }

    #[test]
    fn test_lhvc_round_trip_2() {
        let lhvc = Lhvc {
            configuration_version: 1,
            min_spatial_segmentation_idc: 0x0FFF,
            parallelism_type: 1,
            num_temporal_layers: 0,
            temporal_id_nested: true,
            length_size_minus_one: 1,
            arrays: vec![],
        };

        let mut encoded = Vec::new();
        lhvc.encode(&mut encoded).expect("failed to encode lhvC");
        let mut buf = encoded.as_ref();
        let decoded = Lhvc::decode(&mut buf).unwrap();
        assert_eq!(decoded, lhvc);
    }
}
