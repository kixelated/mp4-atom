use crate::*;

/// SampleGroupDescriptionBox, ISO/IEC 14496-12:2024 Sect 8.9.3
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sgpd {
    pub grouping_type: FourCC,
    pub default_length: Option<u32>,
    pub default_group_description_index: Option<u32>,
    pub static_group_description: bool,
    pub static_mapping: bool,
    pub essential: bool,
    pub entries: Vec<SgpdEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SgpdEntry {
    pub description_length: Option<u32>,
    pub entry: AnySampleGroupEntry,
}

ext!(
    name: Sgpd,
    versions: [0, 1, 2, 3],
    flags: {
        static_group_description = 0,
        static_mapping = 1,
    }
);

impl PartialOrd for SgpdVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (SgpdVersion::V0, SgpdVersion::V0) => Some(std::cmp::Ordering::Equal),
            (SgpdVersion::V0, _) => Some(std::cmp::Ordering::Less),
            (SgpdVersion::V1, SgpdVersion::V0) => Some(std::cmp::Ordering::Greater),
            (SgpdVersion::V1, SgpdVersion::V1) => Some(std::cmp::Ordering::Equal),
            (SgpdVersion::V1, _) => Some(std::cmp::Ordering::Less),
            (SgpdVersion::V2, SgpdVersion::V2) => Some(std::cmp::Ordering::Equal),
            (SgpdVersion::V2, SgpdVersion::V3) => Some(std::cmp::Ordering::Less),
            (SgpdVersion::V2, _) => Some(std::cmp::Ordering::Greater),
            (SgpdVersion::V3, SgpdVersion::V3) => Some(std::cmp::Ordering::Equal),
            (SgpdVersion::V3, _) => Some(std::cmp::Ordering::Greater),
        }
    }
}

impl AtomExt for Sgpd {
    type Ext = SgpdExt;

    const KIND_EXT: FourCC = FourCC::new(b"sgpd");

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: Self::Ext) -> Result<Self> {
        let grouping_type = FourCC::decode(buf)?;
        let default_length = if ext.version >= SgpdVersion::V1 {
            Some(u32::decode(buf)?)
        } else {
            None
        };
        let default_group_description_index = if ext.version >= SgpdVersion::V2 {
            Some(u32::decode(buf)?)
        } else {
            None
        };
        let entry_count = u32::decode(buf)?;
        let mut entries = if let Ok(count) = usize::try_from(entry_count) {
            Vec::with_capacity(count)
        } else {
            Vec::new()
        };
        for _ in 0..entry_count {
            // Spec states: if version>=1 && default_length==0
            // But, default_length.is_some(), if and only if version>=1, so fine to just check for
            // `Some(0)`.
            let description_length = if default_length == Some(0) {
                Some(u32::decode(buf)?)
            } else {
                default_length
            };
            let entry = AnySampleGroupEntry::decode(grouping_type, buf)?;
            entries.push(SgpdEntry {
                description_length,
                entry,
            });
        }
        let static_group_description = ext.static_group_description;
        let static_mapping = ext.static_mapping;
        let essential = ext.version == SgpdVersion::V3;
        Ok(Self {
            grouping_type,
            default_length,
            default_group_description_index,
            static_group_description,
            static_mapping,
            essential,
            entries,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<Self::Ext> {
        let version = if self.essential {
            SgpdVersion::V3
        } else if self.default_group_description_index.is_some() {
            SgpdVersion::V2
        } else if self.default_length.is_some() {
            SgpdVersion::V1
        } else {
            SgpdVersion::V0
        };
        let ext = SgpdExt {
            version,
            static_group_description: self.static_group_description,
            static_mapping: self.static_mapping,
        };
        self.grouping_type.encode(buf)?;
        if let Some(default_length) = self.default_length {
            default_length.encode(buf)?;
        }
        if let Some(default_group_description_index) = self.default_group_description_index {
            default_group_description_index.encode(buf)?;
        }
        (self.entries.len() as u32).encode(buf)?;
        for entry in &self.entries {
            if self.default_length == Some(0) {
                if let Some(description_length) = entry.description_length {
                    description_length.encode(buf)?
                }
            }
            entry.entry.encode(buf)?
        }
        Ok(ext)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnySampleGroupEntry {
    UnknownGroupingType(FourCC, Vec<u8>),
}

impl AnySampleGroupEntry {
    fn decode<B: Buf>(grouping_type: FourCC, buf: &mut B) -> Result<Self> {
        match grouping_type {
            unknown => Ok(Self::UnknownGroupingType(unknown, Vec::decode(buf)?)),
        }
    }

    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        match self {
            Self::UnknownGroupingType(_, bytes) => bytes.encode(buf),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // This example was taken from:
    // https://mpeggroup.github.io/FileFormatConformance/files/published/isobmff/a9-aac-samplegroups-edit.mp4
    //
    // I just extracted the bytes for the sgpd atom location.
    const SIMPLE_SGPD: &[u8] = &[
        0x00, 0x00, 0x00, 0x1A, 0x73, 0x67, 0x70, 0x64, 0x01, 0x00, 0x00, 0x00, 0x72, 0x6F, 0x6C,
        0x6C, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0xFF, 0xFF,
    ];

    #[test]
    fn sgpd_decodes_from_bytes_correctly() {
        let mut buf = Cursor::new(SIMPLE_SGPD);
        let sgpd = Sgpd::decode(&mut buf).expect("sgpd should decode successfully");
        assert_eq!(
            sgpd,
            Sgpd {
                grouping_type: FourCC::from(b"roll"),
                default_length: Some(2),
                default_group_description_index: None,
                static_group_description: false,
                static_mapping: false,
                essential: false,
                entries: vec![SgpdEntry {
                    description_length: Some(2),
                    entry: AnySampleGroupEntry::UnknownGroupingType(
                        FourCC::from(b"roll"),
                        SIMPLE_SGPD[24..].to_vec()
                    )
                }],
            }
        )
    }

    #[test]
    fn sgpd_encodes_from_type_correctly() {
        let sgpd = Sgpd {
            grouping_type: FourCC::from(b"roll"),
            default_length: Some(2),
            default_group_description_index: None,
            static_group_description: false,
            static_mapping: false,
            essential: false,
            entries: vec![SgpdEntry {
                description_length: Some(2),
                entry: AnySampleGroupEntry::UnknownGroupingType(
                    FourCC::from(b"roll"),
                    SIMPLE_SGPD[24..].to_vec(),
                ),
            }],
        };
        let mut buf = Vec::new();
        sgpd.encode(&mut buf).expect("encode should be successful");
        assert_eq!(SIMPLE_SGPD, &buf);
    }
}
