use crate::*;

/// SubSampleInformationBox, ISO/IEC 14496-12:2024 Sect 8.7.7
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Subs {
    pub flags: [u8; 3], // flags are codec specific and not defined directly on subs
    pub entries: Vec<SubsEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SubsEntry {
    pub sample_delta: u32,
    pub subsamples: Vec<SubsSubsample>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SubsSubsample {
    pub size: SubsSubsampleSize,
    pub priority: u8,
    pub discardable: bool,
    pub codec_specific_parameters: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SubsSubsampleSize {
    U16(u16),
    U32(u32),
}
impl Default for SubsSubsampleSize {
    fn default() -> Self {
        // The precedent set by the `ext!` macro is to set V0 as default. Given that for V0 the
        // subsample_size is u16, I set 0u16 as the default.
        Self::U16(0)
    }
}
impl SubsSubsampleSize {
    pub fn value(&self) -> u32 {
        match self {
            Self::U16(n) => u32::from(*n),
            Self::U32(n) => *n,
        }
    }
}

// We can't use the `ext!` macro to implement `Ext` because we need to keep track of all possible
// flags. This is because the box doesn't specify any flags directly, and instead:
// > The semantics of `flags`, if any, shall be supplied for a given coding system. If flags have no
// > semantics for a given coding system, the flags shall be 0.
//
// Therefore, I need to keep all possible flags on the struct, as they may have semantic meaning
// that we can't know solely based on the definition of subs, but the user may require knowledge of
// those flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SubsVersion {
    #[default]
    V0 = 0,
    V1 = 1,
}

impl TryFrom<u8> for SubsVersion {
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::V0),
            1 => Ok(Self::V1),
            _ => Err(Error::UnknownVersion(value)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct SubsExt {
    pub version: SubsVersion,
    pub flags: [u8; 3],
}

impl Ext for SubsExt {
    fn encode(&self) -> Result<u32> {
        Ok((self.version as u32) << 24
            | (self.flags[0] as u32) << 16
            | (self.flags[1] as u32) << 8
            | (self.flags[2] as u32))
    }

    fn decode(v: u32) -> Result<Self> {
        let bytes = v.to_be_bytes();
        let version = SubsVersion::try_from(bytes[0])?;
        let flags = [bytes[1], bytes[2], bytes[3]];
        Ok(Self { version, flags })
    }
}

impl AtomExt for Subs {
    type Ext = SubsExt;

    const KIND_EXT: FourCC = FourCC::new(b"subs");

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: Self::Ext) -> Result<Self> {
        let flags = ext.flags;
        let entry_count = u32::decode(buf)?;
        let mut entries = if let Ok(count) = usize::try_from(entry_count) {
            Vec::with_capacity(count)
        } else {
            Vec::new()
        };
        for _ in 0..entry_count {
            let sample_delta = u32::decode(buf)?;
            let subsample_count = u16::decode(buf)?;
            let mut subsamples = Vec::with_capacity(usize::from(subsample_count));
            for _ in 0..subsample_count {
                let size = if ext.version == SubsVersion::V1 {
                    SubsSubsampleSize::U32(u32::decode(buf)?)
                } else {
                    SubsSubsampleSize::U16(u16::decode(buf)?)
                };
                let priority = u8::decode(buf)?;
                let discardable = u8::decode(buf)? == 1;
                let codec_specific_parameters = buf.slice(4).to_vec();
                buf.advance(4);
                subsamples.push(SubsSubsample {
                    size,
                    priority,
                    discardable,
                    codec_specific_parameters,
                });
            }
            entries.push(SubsEntry {
                sample_delta,
                subsamples,
            });
        }
        Ok(Self { flags, entries })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<Self::Ext> {
        let ext = match &self
            .entries
            .first()
            .and_then(|e| e.subsamples.first())
            .map(|s| &s.size)
        {
            Some(SubsSubsampleSize::U16(_)) => SubsExt {
                version: SubsVersion::V0,
                flags: self.flags,
            },
            Some(SubsSubsampleSize::U32(_)) => SubsExt {
                version: SubsVersion::V1,
                flags: self.flags,
            },
            // Should I store the version somewhere so that I can always decode and encode back to
            // the exact same bytes?
            None => SubsExt {
                version: SubsVersion::default(),
                flags: self.flags,
            },
        };
        (self.entries.len() as u32).encode(buf)?;
        for entry in &self.entries {
            entry.sample_delta.encode(buf)?;
            (entry.subsamples.len() as u16).encode(buf)?;
            for subsample in &entry.subsamples {
                match subsample.size {
                    SubsSubsampleSize::U16(n) => n.encode(buf)?,
                    SubsSubsampleSize::U32(n) => n.encode(buf)?,
                }
                subsample.priority.encode(buf)?;
                if subsample.discardable {
                    1u8.encode(buf)?;
                } else {
                    0u8.encode(buf)?;
                }
                subsample.codec_specific_parameters.encode(buf)?;
            }
        }
        Ok(ext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // This example was taken from:
    // https://mpeggroup.github.io/FileFormatConformance/files/published/uvvu/Solekai007_1920_29_1x1_v7clear.uvu
    //
    // I just extracted the bytes for the subs atom location.
    const SUBS: &[u8] = &[
        0x00, 0x00, 0x00, 0x16, 0x73, 0x75, 0x62, 0x73, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
    ];

    #[test]
    fn subs_decodes_from_bytes_correctly() {
        let mut buf = Cursor::new(SUBS);
        let subs = Subs::decode(&mut buf).expect("subs should decode successfully");
        assert_eq!(
            subs,
            Subs {
                flags: [0, 0, 0],
                entries: vec![SubsEntry {
                    sample_delta: 1,
                    subsamples: vec![],
                }],
            }
        )
    }

    // This example was taken from:
    // https://mpeggroup.github.io/FileFormatConformance/files/published/nalu/hevc/subs_tile_hvc1.mp4
    //
    // I just extracted the bytes for the subs atom location and then modified it to make it
    // shorter.
    //
    // I added this example so that I could test subsample decoding/encoding and therefore also have
    // an encoding test.
    const SUBS_COMPLEX: &[u8] = &[
        0x00, 0x00, 0x00, 0x9C, 0x73, 0x75, 0x62, 0x73, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x0F, 0x97, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x05, 0x52, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0xA3, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x0B, 0x88, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0xA3, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x05, 0xF5, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x4A, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x05, 0x6A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x08, 0x00, 0xC2, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xAB, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x02, 0xE0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xDE, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x05, 0xBE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x4A, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x96, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xF6,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn subs_decodes_from_bytes_and_encodes_to_bytes_correctly_with_more_complex_example() {
        let mut buf = Cursor::new(SUBS_COMPLEX);
        let subs = Subs {
            flags: [0, 0, 2],
            entries: vec![
                SubsEntry {
                    sample_delta: 0,
                    subsamples: vec![
                        SubsSubsample {
                            size: SubsSubsampleSize::U16(3991),
                            priority: 0,
                            discardable: false,
                            codec_specific_parameters: 0u32.to_be_bytes().to_vec(),
                        },
                        SubsSubsample {
                            size: SubsSubsampleSize::U16(1362),
                            priority: 0,
                            discardable: false,
                            codec_specific_parameters: 0u32.to_be_bytes().to_vec(),
                        },
                        SubsSubsample {
                            size: SubsSubsampleSize::U16(1443),
                            priority: 0,
                            discardable: false,
                            codec_specific_parameters: 0u32.to_be_bytes().to_vec(),
                        },
                        SubsSubsample {
                            size: SubsSubsampleSize::U16(2952),
                            priority: 0,
                            discardable: false,
                            codec_specific_parameters: 0u32.to_be_bytes().to_vec(),
                        },
                        SubsSubsample {
                            size: SubsSubsampleSize::U16(1955),
                            priority: 0,
                            discardable: false,
                            codec_specific_parameters: 0u32.to_be_bytes().to_vec(),
                        },
                        SubsSubsample {
                            size: SubsSubsampleSize::U16(1525),
                            priority: 0,
                            discardable: false,
                            codec_specific_parameters: 0u32.to_be_bytes().to_vec(),
                        },
                        SubsSubsample {
                            size: SubsSubsampleSize::U16(842),
                            priority: 0,
                            discardable: false,
                            codec_specific_parameters: 0u32.to_be_bytes().to_vec(),
                        },
                        SubsSubsample {
                            size: SubsSubsampleSize::U16(1386),
                            priority: 0,
                            discardable: false,
                            codec_specific_parameters: 0u32.to_be_bytes().to_vec(),
                        },
                    ],
                },
                SubsEntry {
                    sample_delta: 1,
                    subsamples: vec![
                        SubsSubsample {
                            size: SubsSubsampleSize::U16(194),
                            priority: 0,
                            discardable: false,
                            codec_specific_parameters: 0u32.to_be_bytes().to_vec(),
                        },
                        SubsSubsample {
                            size: SubsSubsampleSize::U16(171),
                            priority: 0,
                            discardable: false,
                            codec_specific_parameters: 0u32.to_be_bytes().to_vec(),
                        },
                        SubsSubsample {
                            size: SubsSubsampleSize::U16(736),
                            priority: 0,
                            discardable: false,
                            codec_specific_parameters: 0u32.to_be_bytes().to_vec(),
                        },
                        SubsSubsample {
                            size: SubsSubsampleSize::U16(734),
                            priority: 0,
                            discardable: false,
                            codec_specific_parameters: 0u32.to_be_bytes().to_vec(),
                        },
                        SubsSubsample {
                            size: SubsSubsampleSize::U16(1470),
                            priority: 0,
                            discardable: false,
                            codec_specific_parameters: 0u32.to_be_bytes().to_vec(),
                        },
                        SubsSubsample {
                            size: SubsSubsampleSize::U16(330),
                            priority: 0,
                            discardable: false,
                            codec_specific_parameters: 0u32.to_be_bytes().to_vec(),
                        },
                        SubsSubsample {
                            size: SubsSubsampleSize::U16(150),
                            priority: 0,
                            discardable: false,
                            codec_specific_parameters: 0u32.to_be_bytes().to_vec(),
                        },
                        SubsSubsample {
                            size: SubsSubsampleSize::U16(1014),
                            priority: 0,
                            discardable: false,
                            codec_specific_parameters: 0u32.to_be_bytes().to_vec(),
                        },
                    ],
                },
            ],
        };
        let decoded = Subs::decode(&mut buf).expect("subs should decode successfully");
        assert_eq!(subs, decoded);
        let mut encoded = Vec::new();
        subs.encode(&mut encoded)
            .expect("encode should be successful");
        assert_eq!(SUBS_COMPLEX, &encoded);
    }
}
