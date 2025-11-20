use super::avc1::AvcSampleEntry;

const AVC3_CODE: u32 = u32::from_be_bytes(*b"avc3");

pub type Avc3 = AvcSampleEntry<{ AVC3_CODE }>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn base(compressor: &str) -> Avc3 {
        Avc3 {
            visual: Visual {
                data_reference_index: 1,
                width: 320,
                height: 240,
                horizresolution: 0x48.into(),
                vertresolution: 0x48.into(),
                frame_count: 1,
                compressor: compressor.into(),
                depth: 24,
            },
            avcc: Avcc {
                configuration_version: 1,
                avc_profile_indication: 100,
                profile_compatibility: 0,
                avc_level_indication: 13,
                length_size: 4,
                sequence_parameter_sets: vec![vec![
                    0x67, 0x64, 0x00, 0x0D, 0xAC, 0xD9, 0x41, 0x41, 0xFA, 0x10, 0x00, 0x00, 0x03,
                    0x00, 0x10, 0x00, 0x00, 0x03, 0x03, 0x20, 0xF1, 0x42, 0x99, 0x60,
                ]],
                picture_parameter_sets: vec![vec![0x68, 0xEB, 0xE3, 0xCB, 0x22, 0xC0]],
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn roundtrip(expected: Avc3) {
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Avc3::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_avc3() {
        roundtrip(base("ya boy"));
    }

    #[test]
    fn test_avc3_with_extras() {
        let mut avc3 = base("they");
        avc3.btrt = Some(Btrt {
            buffer_size_db: 14075,
            max_bitrate: 374288,
            avg_bitrate: 240976,
        });
        avc3.colr = Some(Colr::default());
        avc3.pasp = Some(Pasp {
            h_spacing: 4,
            v_spacing: 3,
        });
        avc3.taic = Some(Taic {
            time_uncertainty: u64::MAX,
            clock_resolution: 1000,
            clock_drift_rate: i32::MAX,
            clock_type: ClockType::CanSync,
        });
        roundtrip(avc3);
    }
}
