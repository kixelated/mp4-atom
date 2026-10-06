use crate::*;

/// An H.264 sample entry like [Avc1], except parameter sets may also be in-band in the samples.
///
/// The `avcC` box is still required (ISO/IEC 14496-15:2022 5.4.2.1.1) but may list no parameter sets.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Avc3 {
    pub visual: Visual,
    pub avcc: Avcc,
    pub btrt: Option<Btrt>,
    pub colr: Option<Colr>,
    pub pasp: Option<Pasp>,
    pub taic: Option<Taic>,
    pub fiel: Option<Fiel>,
}

impl Atom for Avc3 {
    const KIND: FourCC = FourCC::new(b"avc3");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let visual = Visual::decode(buf)?;

        let mut avcc = None;
        let mut btrt = None;
        let mut colr = None;
        let mut pasp = None;
        let mut taic = None;
        let mut fiel = None;
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Avcc(atom) => avcc = atom.into(),
                Any::Btrt(atom) => btrt = atom.into(),
                Any::Colr(atom) => colr = atom.into(),
                Any::Pasp(atom) => pasp = atom.into(),
                Any::Taic(atom) => taic = atom.into(),
                Any::Fiel(atom) => fiel = atom.into(),
                unknown => Self::decode_unknown(&unknown)?,
            }
        }
        skip_trailing_padding(buf);

        Ok(Avc3 {
            visual,
            avcc: avcc.ok_or(Error::MissingBox(Avcc::KIND))?,
            btrt,
            colr,
            pasp,
            taic,
            fiel,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.visual.encode(buf)?;
        self.avcc.encode(buf)?;
        self.btrt.encode(buf)?;
        self.colr.encode(buf)?;
        self.pasp.encode(buf)?;
        self.taic.encode(buf)?;
        self.fiel.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(expected: &Avc3) {
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let decoded = Avc3::decode(&mut buf.as_slice()).unwrap();
        assert_eq!(&decoded, expected);

        // Through the stsd dispatch too, since `Codec::Unknown` refuses to encode.
        let codec = Codec::from(expected.clone());
        let mut buf = Vec::new();
        codec.encode(&mut buf).unwrap();
        assert_eq!(Codec::decode(&mut buf.as_slice()).unwrap(), codec);
    }

    // The avc3 sample entry from the init segment of the BBC Testcard HLS stream:
    // https://vs-dash-ww-rd-live.akamaized.net/pl/testcard2020/192x108p25/media.m3u8
    // Its avcC lists no SPS or PPS; they are all in-band.
    const BBC: &[u8] = &[
        0x00, 0x00, 0x00, 0x88, 0x61, 0x76, 0x63, 0x33, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0xC0, 0x00, 0x6C, 0x00, 0x48, 0x00, 0x00, 0x00, 0x48, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x01, 0x04, 0x68, 0x32, 0x36, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x0F,
        0x61, 0x76, 0x63, 0x43, 0x01, 0x42, 0xC0, 0x15, 0xFF, 0xE0, 0x00, 0x00, 0x00, 0x00, 0x10,
        0x70, 0x61, 0x73, 0x70, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x13, 0x63, 0x6F, 0x6C, 0x72, 0x6E, 0x63, 0x6C, 0x78, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01,
        0x00,
    ];

    #[test]
    fn test_avc3_bbc() {
        let expected = Avc3 {
            visual: Visual {
                data_reference_index: 1,
                width: 192,
                height: 108,
                horizresolution: 0x48.into(),
                vertresolution: 0x48.into(),
                frame_count: 1,
                compressor: "h264".into(),
                depth: 24,
            },
            avcc: Avcc {
                configuration_version: 1,
                avc_profile_indication: 0x42,
                profile_compatibility: 0xC0,
                avc_level_indication: 0x15,
                length_size: 4,
                sequence_parameter_sets: Vec::new(),
                picture_parameter_sets: Vec::new(),
                ext: None,
            },
            colr: Some(Colr::Nclx {
                colour_primaries: 1,
                transfer_characteristics: 1,
                matrix_coefficients: 1,
                full_range_flag: false,
            }),
            pasp: Some(Pasp {
                h_spacing: 1,
                v_spacing: 1,
            }),
            ..Default::default()
        };

        let decoded = Avc3::decode(&mut &BBC[..]).unwrap();
        assert_eq!(decoded, expected);
        roundtrip(&expected);
    }

    #[test]
    fn test_avc3_with_extras() {
        roundtrip(&Avc3 {
            visual: Visual {
                data_reference_index: 1,
                width: 320,
                height: 240,
                horizresolution: 0x48.into(),
                vertresolution: 0x48.into(),
                frame_count: 1,
                compressor: "they".into(),
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
            btrt: Some(Btrt {
                buffer_size_db: 14075,
                max_bitrate: 374288,
                avg_bitrate: 240976,
            }),
            colr: Some(Colr::default()),
            pasp: Some(Pasp {
                h_spacing: 4,
                v_spacing: 3,
            }),
            taic: Some(Taic {
                time_uncertainty: u64::MAX,
                clock_resolution: 1000,
                clock_drift_rate: i32::MAX,
                clock_type: ClockType::CanSync,
            }),
            fiel: Some(Fiel {
                field_count: 2,
                field_order: 0,
            }),
        });
    }
}
