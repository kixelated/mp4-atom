use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hvc1 {
    pub visual: Visual,
    pub hvcc: Hvcc,
    // TODO support SPS/PPS/VPS atoms
    pub btrt: Option<Btrt>,
    pub colr: Option<Colr>,
    pub pasp: Option<Pasp>,
    pub taic: Option<Taic>,
    pub fiel: Option<Fiel>,
    pub ccst: Option<Ccst>,
}

impl Atom for Hvc1 {
    const KIND: FourCC = FourCC::new(b"hvc1");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let visual = Visual::decode(buf)?;

        let mut hvcc = None;
        let mut btrt = None;
        let mut colr = None;
        let mut pasp = None;
        let mut taic = None;
        let mut fiel = None;
        let mut ccst = None;
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Hvcc(atom) => hvcc = atom.into(),
                Any::Btrt(atom) => btrt = atom.into(),
                Any::Colr(atom) => colr = atom.into(),
                Any::Pasp(atom) => pasp = atom.into(),
                Any::Taic(atom) => taic = atom.into(),
                Any::Fiel(atom) => fiel = atom.into(),
                Any::Ccst(atom) => ccst = atom.into(),
                unknown => Self::decode_unknown(&unknown)?,
            }
        }

        Ok(Self {
            visual,
            hvcc: hvcc.ok_or(Error::MissingBox(Hvcc::KIND))?,
            btrt,
            colr,
            pasp,
            taic,
            fiel,
            ccst,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.visual.encode(buf)?;
        self.hvcc.encode(buf)?;
        self.btrt.encode(buf)?;
        self.colr.encode(buf)?;
        self.pasp.encode(buf)?;
        self.taic.encode(buf)?;
        self.fiel.encode(buf)?;
        self.ccst.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // From MPEG File Format Conformance, heif/C001.heif
    const ENCODED_HVC1_HEIF: &[u8] = &[
        0x00, 0x00, 0x00, 0xd2, 0x68, 0x76, 0x63, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x05, 0x00, 0x02, 0xd0, 0x00, 0x48, 0x00, 0x00, 0x00, 0x48, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x01, 0x1f, 0x48, 0x45, 0x56, 0x43, 0x20, 0x43, 0x6f, 0x64, 0x69,
        0x6e, 0x67, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0xff, 0xff, 0x00, 0x00, 0x00, 0x6c,
        0x68, 0x76, 0x63, 0x43, 0x01, 0x01, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x78, 0xf0, 0x00, 0xfc, 0xfd, 0xf8, 0xf8, 0x00, 0x00, 0x0f, 0x03, 0xa0, 0x00, 0x01,
        0x00, 0x18, 0x40, 0x01, 0x0c, 0x01, 0xff, 0xff, 0x01, 0x60, 0x00, 0x00, 0x03, 0x00, 0x00,
        0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x78, 0xf0, 0x24, 0xa1, 0x00, 0x01, 0x00,
        0x1f, 0x42, 0x01, 0x01, 0x01, 0x60, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x03, 0x00, 0x78, 0xa0, 0x02, 0x80, 0x80, 0x2d, 0x1f, 0xe5, 0xf9, 0x24, 0x6d,
        0x9e, 0xd9, 0xa2, 0x00, 0x01, 0x00, 0x07, 0x44, 0x01, 0xc1, 0x90, 0x95, 0x81, 0x12, 0x00,
        0x00, 0x00, 0x10, 0x63, 0x63, 0x73, 0x74, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn test_hvc1_with_ccst() {
        let mut buf = std::io::Cursor::new(ENCODED_HVC1_HEIF);

        let hvc1 = Hvc1::decode(&mut buf).expect("failed to decode hvc1");

        assert_eq!(
            hvc1,
            Hvc1 {
                visual: Visual {
                    data_reference_index: 1,
                    width: 1280,
                    height: 720,
                    horizresolution: 72.into(),
                    vertresolution: 72.into(),
                    frame_count: 1,
                    compressor: "\x1fHEVC Coding".into(),
                    depth: 24
                },
                hvcc: Hvcc {
                    configuration_version: 1,
                    general_profile_space: 0,
                    general_tier_flag: false,
                    general_profile_idc: 1,
                    general_profile_compatibility_flags: [96, 0, 0, 0],
                    general_constraint_indicator_flags: [0, 0, 0, 0, 0, 0],
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
                            completeness: true,
                            nal_unit_type: 32,
                            nalus: vec![vec![
                                64, 1, 12, 1, 255, 255, 1, 96, 0, 0, 3, 0, 0, 3, 0, 0, 3, 0, 0, 3,
                                0, 120, 240, 36
                            ]]
                        },
                        HvcCArray {
                            completeness: true,
                            nal_unit_type: 33,
                            nalus: vec![vec![
                                66, 1, 1, 1, 96, 0, 0, 3, 0, 0, 3, 0, 0, 3, 0, 0, 3, 0, 120, 160,
                                2, 128, 128, 45, 31, 229, 249, 36, 109, 158, 217
                            ]]
                        },
                        HvcCArray {
                            completeness: true,
                            nal_unit_type: 34,
                            nalus: vec![vec![68, 1, 193, 144, 149, 129, 18]]
                        }
                    ]
                },
                btrt: None,
                colr: None,
                pasp: None,
                taic: None,
                fiel: None,
                ccst: Some(Ccst {
                    all_ref_pics_intra: true,
                    intra_pred_used: false,
                    max_ref_per_pic: 0
                })
            }
        );

        let mut encoded = Vec::new();
        hvc1.encode(&mut encoded).expect("failed to encode hvc1");
        assert_eq!(encoded, ENCODED_HVC1_HEIF);
    }
}
