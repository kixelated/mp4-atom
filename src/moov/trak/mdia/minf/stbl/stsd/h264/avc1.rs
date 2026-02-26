use crate::*;

const AVC1_CODE: u32 = u32::from_be_bytes([b'a', b'v', b'c', b'1']);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AvcSampleEntry<const KIND_CODE: u32> {
    pub visual: Visual,
    pub avcc: Avcc,
    pub btrt: Option<Btrt>,
    pub colr: Option<Colr>,
    pub pasp: Option<Pasp>,
    pub taic: Option<Taic>,
    pub fiel: Option<Fiel>,
}

pub type Avc1 = AvcSampleEntry<{ AVC1_CODE }>;

impl<const KIND_CODE: u32> Atom for AvcSampleEntry<KIND_CODE> {
    const KIND: FourCC = FourCC::from_u32(KIND_CODE);

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

        Ok(Self {
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

    #[test]
    fn test_avc1() {
        let expected = Avc1 {
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
            btrt: None,
            colr: None,
            pasp: None,
            taic: None,
            fiel: None,
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Avc1::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_avc1_with_extras() {
        let expected = Avc1 {
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
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Avc1::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
