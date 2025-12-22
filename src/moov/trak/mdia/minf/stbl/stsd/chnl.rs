use crate::*;

// ISO/IEC 23091-3 Audio Channel Position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum AudioChannelPosition {
    FrontLeft = 0,
    FrontRight = 1,
    FrontCenter = 2,
    Lfe1 = 3,
    LeftSurround = 4,
    RightSurround = 5,
    FrontLeftOfCenter = 6,
    FrontRightOfCenter = 7,
    RearLeft = 8,
    RearRight = 9,
    RearCenter = 10,
    SurroundLeft = 11,
    SurroundRight = 12,
    SideLeft = 13,
    SideRight = 14,
    FrontLeftWide = 15,
    FrontRightWide = 16,
    TopFrontLeft = 17,
    TopFrontRight = 18,
    TopFrontCenter = 19,
    TopRearLeft = 20,
    TopRearRight = 21,
    TopRearCenter = 22,
    TopSideLeft = 23,
    TopSideRight = 24,
    TopCenter = 25,
    Lfe2 = 26,
    BottomFrontLeft = 27,
    BottomFrontRight = 28,
    BottomFrontCenter = 29,
    TopSurroundLeft = 30,
    TopSurroundRight = 31,
    // 32-35: reserved
    Lfe3 = 36,
    Leos = 37,
    Reos = 38,
    Hwbcal = 39,
    Hwbcar = 40,
    Lbs = 41,
    Rbs = 42,
    // 45-125: reserved
    // ExplicitPosition is handled separately with SpeakerPosition::Standard.
    // ExplicitPosition = 126, // Followed by azimuth (i16) and elevation (i8)
    Unknown = 127, // Unknown/undefined position (unpositioned)
}

impl AudioChannelPosition {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::FrontLeft),
            1 => Some(Self::FrontRight),
            2 => Some(Self::FrontCenter),
            3 => Some(Self::Lfe1),
            4 => Some(Self::LeftSurround),
            5 => Some(Self::RightSurround),
            6 => Some(Self::FrontLeftOfCenter),
            7 => Some(Self::FrontRightOfCenter),
            8 => Some(Self::RearLeft),
            9 => Some(Self::RearRight),
            10 => Some(Self::RearCenter),
            11 => Some(Self::SurroundLeft),
            12 => Some(Self::SurroundRight),
            13 => Some(Self::SideLeft),
            14 => Some(Self::SideRight),
            15 => Some(Self::FrontLeftWide),
            16 => Some(Self::FrontRightWide),
            17 => Some(Self::TopFrontLeft),
            18 => Some(Self::TopFrontRight),
            19 => Some(Self::TopFrontCenter),
            20 => Some(Self::TopRearLeft),
            21 => Some(Self::TopRearRight),
            22 => Some(Self::TopRearCenter),
            23 => Some(Self::TopSideLeft),
            24 => Some(Self::TopSideRight),
            25 => Some(Self::TopCenter),
            26 => Some(Self::Lfe2),
            27 => Some(Self::BottomFrontLeft),
            28 => Some(Self::BottomFrontRight),
            29 => Some(Self::BottomFrontCenter),
            30 => Some(Self::TopSurroundLeft),
            31 => Some(Self::TopSurroundRight),
            36 => Some(Self::Lfe3),
            37 => Some(Self::Leos),
            38 => Some(Self::Reos),
            39 => Some(Self::Hwbcal),
            40 => Some(Self::Hwbcar),
            41 => Some(Self::Lbs),
            42 => Some(Self::Rbs),
            127 => Some(Self::Unknown),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExplicitSpeakerPosition {
    pub azimuth: i16,
    pub elevation: i8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SpeakerPosition {
    Standard(AudioChannelPosition),
    Explicit(ExplicitSpeakerPosition),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ChannelStructure {
    ExplicitPositions {
        positions: Vec<SpeakerPosition>,
    },
    DefinedLayout {
        layout: u8,
        omitted_channels_map: u64,
    },
}

ext! {
    name: Chnl,
    versions: [0, 1],
    flags: {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Chnl {
    pub channel_structure: Option<ChannelStructure>,
    pub object_count: Option<u8>,
}

struct StreamStructure {
    channel_structured: bool,
    object_structured: bool,
}

impl Chnl {
    fn get_stream_structure<B: Buf>(buf: &mut B) -> Result<StreamStructure> {
        let stream_structure = u8::decode(buf)?;
        let channel_structured = (stream_structure & 0x01) != 0;
        let object_structured = (stream_structure & 0x02) != 0;

        Ok(StreamStructure {
            channel_structured,
            object_structured,
        })
    }

    fn decode_channel_structure<B: Buf>(
        buf: &mut B,
        channel_count: Option<u16>,
        object_structured: bool,
    ) -> Result<Option<ChannelStructure>> {
        let defined_layout = u8::decode(buf)?;

        if defined_layout == 0 {
            // Explicit positions
            let mut positions = Vec::new();

            match channel_count {
                // When channel count is known, read exactly that many positions
                Some(chnl_count) => {
                    positions.reserve(chnl_count as usize);
                    for _ in 0..chnl_count {
                        positions.push(Self::decode_speaker_position(buf)?);
                    }
                }
                // When channel count is unknown, read until end of buffer
                None => {
                    let reserved_bytes = if object_structured { 1 } else { 0 };
                    while buf.remaining() > reserved_bytes {
                        positions.push(Self::decode_speaker_position(buf)?);
                    }
                }
            }

            Ok(Some(ChannelStructure::ExplicitPositions { positions }))
        } else {
            // Pre-defined layout with omitted channels map
            let omitted_channels_map = u64::decode(buf)?;
            Ok(Some(ChannelStructure::DefinedLayout {
                layout: defined_layout,
                omitted_channels_map,
            }))
        }
    }

    fn decode_speaker_position<B: Buf>(buf: &mut B) -> Result<SpeakerPosition> {
        let speaker_position = u8::decode(buf)?;

        if speaker_position == 126 {
            // Explicit position
            let azimuth = i16::decode(buf)?;
            let elevation = i8::decode(buf)?;
            Ok(SpeakerPosition::Explicit(ExplicitSpeakerPosition {
                azimuth,
                elevation,
            }))
        } else if let Some(pos) = AudioChannelPosition::from_u8(speaker_position) {
            Ok(SpeakerPosition::Standard(pos))
        } else {
            Err(Error::Unsupported("invalid speaker position"))
        }
    }

    // Allow Pcm box to decode the channel layout by using the channel
    // count information from audio sample entry when available.
    pub fn decode_body_with_channel_count<B: Buf>(buf: &mut B, channel_count: u16) -> Result<Self> {
        let version_and_flags = u32::decode(buf)?;
        let version = (version_and_flags >> 24) as u8;
        let flags = version_and_flags & 0xFFFFFF;

        if version != 0 {
            return Err(Error::Unsupported("version 1 not supported"));
        }

        if flags != 0 {
            return Err(Error::Unsupported("chnl box with non-zero flags"));
        }

        let stream_structure = Self::get_stream_structure(buf)?;

        let channel_structure = if stream_structure.channel_structured {
            Self::decode_channel_structure(
                buf,
                Some(channel_count),
                stream_structure.object_structured,
            )?
        } else {
            None
        };

        let object_count = if stream_structure.object_structured {
            Some(u8::decode(buf)?)
        } else {
            None
        };

        Ok(Self {
            channel_structure,
            object_count,
        })
    }
}

// ISO/IEC 14496-12:2022 Section 12.2.4
impl AtomExt for Chnl {
    const KIND_EXT: FourCC = FourCC::new(b"chnl");

    type Ext = ChnlExt;

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: ChnlExt) -> Result<Self> {
        if ext.version == ChnlVersion::V1 {
            return Err(Error::Unsupported("version 1 not supported"));
        }

        let stream_structure = Self::get_stream_structure(buf)?;

        let channel_structure = if stream_structure.channel_structured {
            Self::decode_channel_structure(buf, None, stream_structure.object_structured)?
        } else {
            None
        };

        let object_count = if stream_structure.object_structured {
            Some(u8::decode(buf)?)
        } else {
            None
        };

        Ok(Self {
            channel_structure,
            object_count,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<ChnlExt> {
        let channel_structured = self.channel_structure.is_some();
        let object_structured = self.object_count.is_some();
        let mut stream_structure = 0u8;

        if channel_structured {
            stream_structure |= 0x01;
        }
        if object_structured {
            stream_structure |= 0x02;
        }

        stream_structure.encode(buf)?;

        if let Some(ref structure) = self.channel_structure {
            match structure {
                ChannelStructure::ExplicitPositions { positions } => {
                    (0u8).encode(buf)?; // defined_layout = 0

                    for position in positions {
                        match position {
                            SpeakerPosition::Standard(pos) => {
                                (*pos as u8).encode(buf)?;
                            }
                            SpeakerPosition::Explicit(explicit) => {
                                (126u8).encode(buf)?;
                                explicit.azimuth.encode(buf)?;
                                explicit.elevation.encode(buf)?;
                            }
                        }
                    }
                }
                ChannelStructure::DefinedLayout {
                    layout,
                    omitted_channels_map,
                } => {
                    layout.encode(buf)?;
                    omitted_channels_map.encode(buf)?;
                }
            }
        }

        if let Some(object_count) = self.object_count {
            object_count.encode(buf)?;
        }

        Ok(ChnlExt {
            version: ChnlVersion::V0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chnl_explicit_positions() {
        let chnl = Chnl {
            channel_structure: Some(ChannelStructure::ExplicitPositions {
                positions: vec![
                    SpeakerPosition::Standard(AudioChannelPosition::FrontLeft),
                    SpeakerPosition::Standard(AudioChannelPosition::FrontRight),
                ],
            }),
            object_count: None,
        };

        let mut buf = Vec::new();
        chnl.encode(&mut buf).unwrap();

        let decoded = Chnl::decode(&mut &buf[..]).unwrap();
        assert_eq!(chnl, decoded);
    }

    #[test]
    fn test_chnl_defined_layout() {
        let chnl = Chnl {
            channel_structure: Some(ChannelStructure::DefinedLayout {
                layout: 2, // Stereo
                omitted_channels_map: 0,
            }),
            object_count: None,
        };

        let mut buf = Vec::new();
        chnl.encode(&mut buf).unwrap();

        let decoded = Chnl::decode(&mut &buf[..]).unwrap();
        assert_eq!(chnl, decoded);
    }

    #[test]
    fn test_chnl_with_explicit_position() {
        let chnl = Chnl {
            channel_structure: Some(ChannelStructure::ExplicitPositions {
                positions: vec![
                    SpeakerPosition::Standard(AudioChannelPosition::FrontLeft),
                    SpeakerPosition::Explicit(ExplicitSpeakerPosition {
                        azimuth: 45,
                        elevation: 10,
                    }),
                ],
            }),
            object_count: None,
        };

        let mut buf = Vec::new();
        chnl.encode(&mut buf).unwrap();

        let decoded = Chnl::decode(&mut &buf[..]).unwrap();
        assert_eq!(chnl, decoded);
    }

    #[test]
    fn test_chnl_with_both_channel_and_object() {
        let chnl = Chnl {
            channel_structure: Some(ChannelStructure::ExplicitPositions {
                positions: vec![
                    SpeakerPosition::Standard(AudioChannelPosition::FrontLeft),
                    SpeakerPosition::Standard(AudioChannelPosition::FrontRight),
                ],
            }),
            object_count: Some(2),
        };

        let mut buf = Vec::new();
        chnl.encode(&mut buf).unwrap();

        let decoded = Chnl::decode(&mut &buf[..]).unwrap();
        assert_eq!(chnl, decoded);
    }

    #[test]
    fn test_chnl_decode_with_channel_count() {
        let chnl = Chnl {
            channel_structure: Some(ChannelStructure::ExplicitPositions {
                positions: vec![
                    SpeakerPosition::Standard(AudioChannelPosition::FrontLeft),
                    SpeakerPosition::Standard(AudioChannelPosition::FrontRight),
                ],
            }),
            object_count: Some(2),
        };

        let mut buf = Vec::new();
        chnl.encode(&mut buf).unwrap();

        // Skip the atom header (4 bytes: size + 4 bytes: fourcc)
        // but keep the FullBox header (4 bytes: version + flags) and body
        let body_start = 8;
        let mut body_buf = &buf[body_start..];

        let decoded = Chnl::decode_body_with_channel_count(&mut body_buf, 2).unwrap();
        assert_eq!(chnl, decoded);
    }
}
