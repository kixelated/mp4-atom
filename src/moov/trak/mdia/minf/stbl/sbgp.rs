use crate::*;

/// SampleToGroupBox, ISO/IEC 14496-12:2024 Sect 8.9.2
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sbgp {
    pub grouping_type: FourCC,
    pub grouping_type_parameter: Option<u32>,
    pub entries: Vec<SbgpEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SbgpEntry {
    pub sample_count: u32,
    pub group_description_index: u32,
}

ext! {
    name: Sbgp,
    versions: [0, 1],
    flags: {}
}

impl AtomExt for Sbgp {
    type Ext = SbgpExt;

    const KIND_EXT: FourCC = FourCC::new(b"sbgp");

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: Self::Ext) -> Result<Self> {
        let grouping_type = FourCC::decode(buf)?;
        let grouping_type_parameter = if ext.version == SbgpVersion::V1 {
            Some(u32::decode(buf)?)
        } else {
            None
        };
        let entry_count = u32::decode(buf)?;
        // Use with_capacity to reduce the number of times that the Vec will reallocate to grow for
        // more entries; however, limit to a max of 1024 entries to start with, as the `entry_count`
        // is a number defined from outside data (that is being decoded), and so is an attack vector
        // if a malicious actor set a very high number.
        let mut entries = Vec::with_capacity((entry_count as usize).min(1024));
        for _ in 0..entry_count {
            let sample_count = u32::decode(buf)?;
            let group_description_index = u32::decode(buf)?;
            entries.push(SbgpEntry {
                sample_count,
                group_description_index,
            });
        }
        Ok(Self {
            grouping_type,
            grouping_type_parameter,
            entries,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<Self::Ext> {
        let ext = if self.grouping_type_parameter.is_some() {
            SbgpExt {
                version: SbgpVersion::V1,
            }
        } else {
            SbgpExt {
                version: SbgpVersion::V0,
            }
        };
        self.grouping_type.encode(buf)?;
        if let Some(grouping_type_parameter) = self.grouping_type_parameter {
            grouping_type_parameter.encode(buf)?;
        }
        (self.entries.len() as u32).encode(buf)?;
        for entry in &self.entries {
            entry.sample_count.encode(buf)?;
            entry.group_description_index.encode(buf)?
        }
        Ok(ext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // This example was taken from:
    // https://mpeggroup.github.io/FileFormatConformance/files/published/isobmff/a9-aac-samplegroups-edit.mp4
    //
    // I just extracted the bytes for the sbgp atom location.
    const SIMPLE_SBGP: &[u8] = &[
        0x00, 0x00, 0x00, 0x1C, 0x73, 0x62, 0x67, 0x70, 0x00, 0x00, 0x00, 0x00, 0x72, 0x6F, 0x6C,
        0x6C, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x30, 0x00, 0x00, 0x00, 0x01,
    ];

    #[test]
    fn sbgp_decodes_from_bytes_correctly() {
        let mut buf = Cursor::new(SIMPLE_SBGP);
        let sbgp = Sbgp::decode(&mut buf).expect("sbgp should decode successfully");
        assert_eq!(
            sbgp,
            Sbgp {
                grouping_type: FourCC::from(b"roll"),
                grouping_type_parameter: None,
                entries: vec![SbgpEntry {
                    sample_count: 48,
                    group_description_index: 1,
                }],
            }
        )
    }

    #[test]
    fn sbgp_encodes_from_type_correctly() {
        let sbgp = Sbgp {
            grouping_type: FourCC::from(b"roll"),
            grouping_type_parameter: None,
            entries: vec![SbgpEntry {
                sample_count: 48,
                group_description_index: 1,
            }],
        };
        let mut buf = Vec::new();
        sbgp.encode(&mut buf).expect("encode should be successful");
        assert_eq!(SIMPLE_SBGP, &buf);
    }
}
