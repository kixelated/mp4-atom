use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AuxInfo {
    pub aux_info_type: FourCC,
    pub aux_info_type_parameter: u32,
}

ext! {
    name: Saiz,
    versions: [0],
    flags: {
        aux_info_type_present  = 0,
    }
}

ext! {
    name: Saio,
    versions: [0, 1],
    flags: {
        aux_info_type_present  = 0,
    }
}

/// Sample AuxiliaryInformationSizesBox, ISO/IEC 14496-12:2022 Sect 8.7.8
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Saiz {
    pub aux_info: Option<AuxInfo>,
    pub default_sample_info_size: u8,
    pub sample_count: u32,
    pub sample_info_size: Vec<u8>,
}

impl AtomExt for Saiz {
    type Ext = SaizExt;

    const KIND_EXT: FourCC = FourCC::new(b"saiz");

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: SaizExt) -> Result<Self> {
        let mut aux_info = None;
        if ext.aux_info_type_present {
            let aux_info_type = FourCC::decode(buf)?;
            let aux_info_type_parameter = u32::decode(buf)?;
            aux_info = Some(AuxInfo {
                aux_info_type,
                aux_info_type_parameter,
            });
        }
        let default_sample_info_size = u8::decode(buf)?;
        let sample_count = u32::decode(buf)?;
        if default_sample_info_size == 0 {
            let mut sample_info_size = Vec::with_capacity(sample_count as usize);
            for _ in 0..sample_count {
                sample_info_size.push(u8::decode(buf)?);
            }
            Ok(Saiz {
                aux_info,
                default_sample_info_size,
                sample_count,
                sample_info_size,
            })
        } else {
            Ok(Saiz {
                aux_info,
                default_sample_info_size,
                sample_count,
                sample_info_size: vec![],
            })
        }
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<SaizExt> {
        let ext = SaizExt {
            version: SaizVersion::V0,
            aux_info_type_present: self.aux_info.is_some(),
        };
        if let Some(aux_info) = &self.aux_info {
            aux_info.aux_info_type.encode(buf)?;
            aux_info.aux_info_type_parameter.encode(buf)?;
        }
        self.default_sample_info_size.encode(buf)?;
        self.sample_count.encode(buf)?;
        if self.default_sample_info_size == 0 {
            for size in &self.sample_info_size {
                size.encode(buf)?;
            }
        }
        Ok(ext)
    }
}

/// SampleAuxiliaryInformationOffsetsBox, ISO/IEC 14496-12:2022 Sect 8.7.9
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Saio {
    pub aux_info: Option<AuxInfo>,
    pub offsets: Vec<u64>,
}

impl AtomExt for Saio {
    type Ext = SaioExt;

    const KIND_EXT: FourCC = FourCC::new(b"saio");

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: SaioExt) -> Result<Self> {
        let mut aux_info = None;
        if ext.aux_info_type_present {
            let aux_info_type = FourCC::decode(buf)?;
            let aux_info_type_parameter = u32::decode(buf)?;
            aux_info = Some(AuxInfo {
                aux_info_type,
                aux_info_type_parameter,
            });
        }
        let entry_count = u32::decode(buf)?;
        let mut offsets = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            if ext.version == SaioVersion::V0 {
                let offset = u32::decode(buf)? as u64;
                offsets.push(offset);
            } else if ext.version == SaioVersion::V1 {
                let offset = u64::decode(buf)?;
                offsets.push(offset);
            }
        }
        Ok(Saio { aux_info, offsets })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<SaioExt> {
        let mut version: SaioVersion = SaioVersion::V0;
        for offset in &self.offsets {
            if *offset > (u32::MAX as u64) {
                version = SaioVersion::V1;
                break;
            }
        }

        let ext = SaioExt {
            version,
            aux_info_type_present: self.aux_info.is_some(),
        };
        if let Some(aux_info) = &self.aux_info {
            aux_info.aux_info_type.encode(buf)?;
            aux_info.aux_info_type_parameter.encode(buf)?;
        }
        let entry_count: u32 = self.offsets.len() as u32;
        entry_count.encode(buf)?;
        if ext.version == SaioVersion::V0 {
            for i in 0..self.offsets.len() {
                let offset: u32 = self.offsets[i] as u32;
                offset.encode(buf)?;
            }
        } else {
            for offset in &self.offsets {
                offset.encode(buf)?;
            }
        }
        Ok(ext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENCODED_SAIZ: &[u8] = &[
        0x00, 0x00, 0x00, 0x11, 0x73, 0x61, 0x69, 0x7a, 0x00, 0x00, 0x00, 0x00, 0x46, 0x00, 0x00,
        0x00, 0x32,
    ];

    const ENCODED_SAIZ_CENC: &[u8] = &[
        0x00, 0x00, 0x03, 0x07, 0x73, 0x61, 0x69, 0x7a, 0x00, 0x00, 0x00, 0x01, 0x63, 0x65, 0x6e,
        0x63, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xee, 0x1e, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x24, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x24,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x24, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x24, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x24,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x24, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x24, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x24,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x24, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x24, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x24,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x24, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x24, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x24,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x24, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x24, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x24,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x24, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x24, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x24,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x24, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x24, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x24,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x24, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x24, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x24,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x24, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x24, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x24,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
        0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
    ];

    const ENCODED_SAIO: &[u8] = &[
        0x00, 0x00, 0x00, 0x14, 0x73, 0x61, 0x69, 0x6f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x04, 0xbc,
    ];

    const ENCODED_SAIO_CENC: &[u8] = &[
        0x00, 0x00, 0x00, 0x1c, 0x73, 0x61, 0x69, 0x6f, 0x00, 0x00, 0x00, 0x01, 0x63, 0x65, 0x6e,
        0x63, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x04, 0x8e,
    ];

    #[test]
    fn test_saiz_decode() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_SAIZ);

        let saiz = Saiz::decode(buf).expect("failed to decode saiz");

        assert_eq!(
            saiz,
            Saiz {
                aux_info: None,
                default_sample_info_size: 70,
                sample_count: 50,
                sample_info_size: vec![]
            },
        );
    }

    #[test]
    fn test_saiz_encode() {
        let saiz = Saiz {
            aux_info: None,
            default_sample_info_size: 70,
            sample_count: 50,
            sample_info_size: vec![],
        };

        let mut buf = Vec::new();
        saiz.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_SAIZ);
    }

    #[test]
    fn test_saiz_encode_cenc() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_SAIZ_CENC);

        let saiz = Saiz::decode(buf).expect("failed to decode saiz");

        assert_eq!(
            saiz,
            Saiz {
                aux_info: Some(AuxInfo {
                    aux_info_type: FourCC::new(b"cenc"),
                    aux_info_type_parameter: 0,
                }),
                default_sample_info_size: 0,
                sample_count: 750,
                sample_info_size: vec![
                    30, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                    24, 24, 24, 24, 24, 24, 24, 24, 24, 24
                ]
            },
        );
    }

    #[test]
    fn test_saiz_decode_cenc() {
        let saiz = Saiz {
            aux_info: Some(AuxInfo {
                aux_info_type: FourCC::new(b"cenc"),
                aux_info_type_parameter: 0,
            }),
            default_sample_info_size: 0,
            sample_count: 750,
            sample_info_size: vec![
                30, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 36, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
                24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
            ],
        };

        let mut buf = Vec::new();
        saiz.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_SAIZ_CENC);
    }

    #[test]
    fn test_saio_decode() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_SAIO);

        let saio = Saio::decode(buf).expect("failed to decode saio");

        assert_eq!(
            saio,
            Saio {
                aux_info: None,
                offsets: vec![1212],
            }
        );
    }

    #[test]
    fn test_saio_encode() {
        let saio = Saio {
            aux_info: None,
            offsets: vec![1212],
        };

        let mut buf = Vec::new();
        saio.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_SAIO);
    }

    #[test]
    fn test_saio_decode_cenc() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_SAIO_CENC);

        let saio = Saio::decode(buf).expect("failed to decode saio");

        assert_eq!(
            saio,
            Saio {
                aux_info: Some(AuxInfo {
                    aux_info_type: FourCC::new(b"cenc"),
                    aux_info_type_parameter: 0
                }),
                offsets: vec![1166],
            }
        );
    }

    #[test]
    fn test_saio_encode_cenc() {
        let saio = Saio {
            aux_info: Some(AuxInfo {
                aux_info_type: FourCC::new(b"cenc"),
                aux_info_type_parameter: 0,
            }),
            offsets: vec![1166],
        };

        let mut buf = Vec::new();
        saio.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_SAIO_CENC);
    }
}
