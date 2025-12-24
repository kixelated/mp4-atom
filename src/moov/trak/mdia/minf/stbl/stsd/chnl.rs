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

// Pre-defined channel layouts
const CHANNEL_LAYOUTS: &[&[AudioChannelPosition]] = &[
    // 0
    &[],
    // 1
    &[AudioChannelPosition::FrontCenter],
    // 2
    &[
        AudioChannelPosition::FrontLeft,
        AudioChannelPosition::FrontRight,
    ],
    // 3
    &[
        AudioChannelPosition::FrontCenter,
        AudioChannelPosition::FrontLeft,
        AudioChannelPosition::FrontRight,
    ],
    // 4
    &[
        AudioChannelPosition::FrontCenter,
        AudioChannelPosition::FrontLeft,
        AudioChannelPosition::FrontRight,
        AudioChannelPosition::RearCenter,
    ],
    // 5
    &[
        AudioChannelPosition::FrontCenter,
        AudioChannelPosition::FrontLeft,
        AudioChannelPosition::FrontRight,
        AudioChannelPosition::SideLeft,
        AudioChannelPosition::SideRight,
    ],
    // 6
    &[
        AudioChannelPosition::FrontCenter,
        AudioChannelPosition::FrontLeft,
        AudioChannelPosition::FrontRight,
        AudioChannelPosition::SideLeft,
        AudioChannelPosition::SideRight,
        AudioChannelPosition::Lfe1,
    ],
    // 7
    &[
        AudioChannelPosition::FrontCenter,
        AudioChannelPosition::FrontLeftOfCenter,
        AudioChannelPosition::FrontRightOfCenter,
        AudioChannelPosition::FrontLeft,
        AudioChannelPosition::FrontRight,
        AudioChannelPosition::SideLeft,
        AudioChannelPosition::SideRight,
        AudioChannelPosition::Lfe1,
    ],
    // 8
    &[],
    // 9
    &[
        AudioChannelPosition::FrontLeft,
        AudioChannelPosition::FrontRight,
        AudioChannelPosition::RearCenter,
    ],
    // 10
    &[
        AudioChannelPosition::FrontLeft,
        AudioChannelPosition::FrontRight,
        AudioChannelPosition::SideLeft,
        AudioChannelPosition::SideRight,
    ],
    // 11
    &[
        AudioChannelPosition::FrontCenter,
        AudioChannelPosition::FrontLeft,
        AudioChannelPosition::FrontRight,
        AudioChannelPosition::SideLeft,
        AudioChannelPosition::SideRight,
        AudioChannelPosition::RearCenter,
        AudioChannelPosition::Lfe1,
    ],
    // 12
    &[
        AudioChannelPosition::FrontCenter,
        AudioChannelPosition::FrontLeft,
        AudioChannelPosition::FrontRight,
        AudioChannelPosition::SideLeft,
        AudioChannelPosition::SideRight,
        AudioChannelPosition::RearLeft,
        AudioChannelPosition::RearRight,
        AudioChannelPosition::Lfe1,
    ],
    // 13
    &[
        AudioChannelPosition::FrontCenter,
        AudioChannelPosition::FrontLeftOfCenter,
        AudioChannelPosition::FrontRightOfCenter,
        AudioChannelPosition::FrontLeft,
        AudioChannelPosition::FrontRight,
        AudioChannelPosition::SurroundLeft,
        AudioChannelPosition::SurroundRight,
        AudioChannelPosition::RearLeft,
        AudioChannelPosition::RearRight,
        AudioChannelPosition::RearCenter,
        AudioChannelPosition::Lfe1,
        AudioChannelPosition::Lfe2,
        AudioChannelPosition::TopFrontCenter,
        AudioChannelPosition::TopFrontLeft,
        AudioChannelPosition::TopFrontRight,
        AudioChannelPosition::TopSideLeft,
        AudioChannelPosition::TopSideRight,
        AudioChannelPosition::TopCenter,
        AudioChannelPosition::TopRearLeft,
        AudioChannelPosition::TopRearRight,
        AudioChannelPosition::TopRearCenter,
        AudioChannelPosition::BottomFrontCenter,
        AudioChannelPosition::BottomFrontLeft,
        AudioChannelPosition::BottomFrontRight,
    ],
    // 14
    &[
        AudioChannelPosition::FrontCenter,
        AudioChannelPosition::FrontLeft,
        AudioChannelPosition::FrontRight,
        AudioChannelPosition::SideLeft,
        AudioChannelPosition::SideRight,
        AudioChannelPosition::Lfe1,
        AudioChannelPosition::TopFrontLeft,
        AudioChannelPosition::TopFrontRight,
    ],
];

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
        omitted_channels_map: Option<u64>,
        channel_order_definition: Option<u8>,
    },
}

impl ChannelStructure {
    fn channel_count(&self) -> u8 {
        match self {
            Self::ExplicitPositions { positions } => positions.len() as u8,
            Self::DefinedLayout { layout, .. } => CHANNEL_LAYOUTS
                .get(*layout as usize)
                .map(|l| l.len())
                .unwrap_or(0) as u8,
        }
    }
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
    pub format_ordering: Option<u8>,
    pub base_channel_count: Option<u8>,
}

#[derive(Debug)]
struct StreamStructure {
    channel_structured: bool,
    object_structured: bool,
}

impl Chnl {
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

    fn decode_stream_structure_v0<B: Buf>(buf: &mut B) -> Result<StreamStructure> {
        let stream_structure = u8::decode(buf)?;
        let channel_structured = (stream_structure & 0x01) != 0;
        let object_structured = (stream_structure & 0x02) != 0;

        Ok(StreamStructure {
            channel_structured,
            object_structured,
        })
    }

    fn decode_stream_structure_v1<B: Buf>(buf: &mut B) -> Result<(StreamStructure, u8)> {
        let byte = u8::decode(buf)?;
        let stream_structure = (byte >> 4) & 0x0F;
        let format_ordering = byte & 0x0F;

        let channel_structured = (stream_structure & 0x01) != 0;
        let object_structured = (stream_structure & 0x02) != 0;

        Ok((
            StreamStructure {
                channel_structured,
                object_structured,
            },
            format_ordering,
        ))
    }

    fn decode_channel_structure_v0<B: Buf>(
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
                // Ideally the channel layout box will be decoded along
                // with the audio sample entry which should call the
                // decode_body_with_channel_count variant. So we should
                // not be here except for the synthetic test cases below.
                None => {
                    // Workaround: When channel count is unknown, read until
                    // end of buffer.
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
                omitted_channels_map: Some(omitted_channels_map),
                channel_order_definition: None,
            }))
        }
    }

    fn decode_channel_structure_v1<B: Buf>(buf: &mut B) -> Result<Option<ChannelStructure>> {
        let defined_layout = u8::decode(buf)?;

        if defined_layout == 0 {
            // Explicit positions with explicit channel count
            let layout_channel_count = u8::decode(buf)?;
            let mut positions = Vec::with_capacity(layout_channel_count as usize);

            for _ in 0..layout_channel_count {
                positions.push(Self::decode_speaker_position(buf)?);
            }

            Ok(Some(ChannelStructure::ExplicitPositions { positions }))
        } else {
            // Pre-defined layout
            let byte = u8::decode(buf)?;
            let channel_order_definition = (byte >> 1) & 0x07;
            let omitted_channels_present = (byte & 0x01) != 0;

            // ISO/IEC 14496-12:2022 Section 12.2.4.3
            if channel_order_definition > 4 {
                return Err(Error::Unsupported("invalid channel order definition"));
            }

            let omitted_channels_map = if omitted_channels_present {
                Some(u64::decode(buf)?)
            } else {
                None
            };

            Ok(Some(ChannelStructure::DefinedLayout {
                layout: defined_layout,
                omitted_channels_map,
                channel_order_definition: Some(channel_order_definition),
            }))
        }
    }

    fn get_object_count_v1(
        channel_structure: Option<&ChannelStructure>,
        base_channel_count: u8,
    ) -> Option<u8> {
        // ISO/IEC 14496-12:2022 Section 12.2.4.3
        // objectCount = baseChannelCount - channel count of channel structure.
        channel_structure
            .map(|cs| base_channel_count.saturating_sub(cs.channel_count()))
            .filter(|&count| count != 0)
    }

    fn encode_object_structure_v1<B: BufMut>(
        buf: &mut B,
        channel_structure: Option<&ChannelStructure>,
        base_channel_count: u8,
    ) -> Result<()> {
        channel_structure
            // ISO/IEC 14496-12:2022 Section 12.2.4.3
            // objectCount = baseChannelCount - channel count of channel structure.
            .map(|cs| base_channel_count.saturating_sub(cs.channel_count()))
            .filter(|&count| count > 0)
            .map(|count| count.encode(buf))
            .transpose()?;

        Ok(())
    }

    fn encode_channel_structure_v0<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let Some(ref structure) = self.channel_structure else {
            return Ok(());
        };

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
                ..
            } => {
                layout.encode(buf)?;

                let map = omitted_channels_map.ok_or_else(|| {
                    Error::Unsupported("omitted_channels_map required for version 0 defined layout")
                })?;
                map.encode(buf)?;
            }
        }

        Ok(())
    }

    fn encode_channel_structure_v1<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let Some(ref structure) = self.channel_structure else {
            return Ok(());
        };

        match structure {
            ChannelStructure::ExplicitPositions { positions } => {
                (0u8).encode(buf)?; // defined_layout = 0
                (positions.len() as u8).encode(buf)?; // layout_channel_count

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
                channel_order_definition,
            } => {
                layout.encode(buf)?;

                let channel_order_def = channel_order_definition.unwrap_or(0);
                let omitted_present = omitted_channels_map.is_some();

                let combined = (channel_order_def << 1) | (if omitted_present { 1 } else { 0 });
                combined.encode(buf)?;

                if let Some(map) = omitted_channels_map {
                    map.encode(buf)?;
                }
            }
        }

        Ok(())
    }

    fn decode_body_v0<B: Buf>(buf: &mut B, channel_count: Option<u16>) -> Result<Self> {
        let stream_structure = Self::decode_stream_structure_v0(buf)?;

        let channel_structure = if stream_structure.channel_structured {
            Self::decode_channel_structure_v0(
                buf,
                channel_count,
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
            format_ordering: None,
            base_channel_count: None,
        })
    }

    fn decode_body_v1<B: Buf>(buf: &mut B) -> Result<Self> {
        let (stream_structure, format_ordering) = Self::decode_stream_structure_v1(buf)?;
        let base_channel_count = u8::decode(buf)?;

        let channel_structure = if stream_structure.channel_structured {
            Self::decode_channel_structure_v1(buf)?
        } else {
            None
        };

        // ISO/IEC 14496-12:2022 Section 12.2.4.3
        // objectCount = baseChannelCount - channel count of channel structure.
        let computed_object_count = stream_structure
            .object_structured
            .then(|| Self::get_object_count_v1(channel_structure.as_ref(), base_channel_count))
            .flatten();

        let decoded_object_count = if stream_structure.object_structured {
            Some(u8::decode(buf)?)
        } else {
            None
        };

        if stream_structure.object_structured && computed_object_count != decoded_object_count {
            return Err(Error::Unsupported(
                "computed object count != decoded object count",
            ));
        }

        Ok(Self {
            channel_structure,
            object_count: decoded_object_count,
            format_ordering: Some(format_ordering),
            base_channel_count: Some(base_channel_count),
        })
    }

    // Allow Pcm box to decode the channel layout by using the channel
    // count information from audio sample entry when available.
    pub fn decode_body_with_channel_count<B: Buf>(buf: &mut B, channel_count: u16) -> Result<Self> {
        let version_and_flags = u32::decode(buf)?;
        let version = (version_and_flags >> 24) as u8;
        let flags = version_and_flags & 0xFFFFFF;

        if flags != 0 {
            return Err(Error::Unsupported("chnl box with non-zero flags"));
        }

        match version {
            0 => Self::decode_body_v0(buf, Some(channel_count)),
            1 => Self::decode_body_v1(buf),
            _ => Err(Error::Unsupported("invalid version")),
        }
    }
}

// ISO/IEC 14496-12:2022 Section 12.2.4
impl AtomExt for Chnl {
    const KIND_EXT: FourCC = FourCC::new(b"chnl");

    type Ext = ChnlExt;

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: ChnlExt) -> Result<Self> {
        match ext.version {
            ChnlVersion::V0 => Self::decode_body_v0(buf, None),
            ChnlVersion::V1 => Self::decode_body_v1(buf),
        }
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<ChnlExt> {
        // Determine version based on presence of v1-specific fields
        let version = if self.format_ordering.is_some() && self.base_channel_count.is_some() {
            ChnlVersion::V1
        } else {
            ChnlVersion::V0
        };

        match version {
            ChnlVersion::V0 => {
                let mut stream_structure = 0u8;

                if self.channel_structure.is_some() {
                    stream_structure |= 0x01;
                }

                if self.object_count.is_some() {
                    stream_structure |= 0x02;
                }

                stream_structure.encode(buf)?;

                self.encode_channel_structure_v0(buf)?;

                if let Some(object_count) = self.object_count {
                    object_count.encode(buf)?;
                }
            }
            ChnlVersion::V1 => {
                let base_channel_count = self.base_channel_count.unwrap();
                let format_ordering = self.format_ordering.unwrap_or(1);

                // ISO/IEC 14496-12:2022 Section 12.2.4.3
                if !(0..=2).contains(&format_ordering) {
                    return Err(Error::Unsupported("format ordering must be 0, 1 or 2"));
                }

                let object_count =
                    Self::get_object_count_v1(self.channel_structure.as_ref(), base_channel_count);
                let object_structured = object_count.is_some();
                let channel_structured = self.channel_structure.is_some();

                let mut stream_structure = 0u8;
                if channel_structured {
                    stream_structure |= 0x01;
                }
                if object_structured {
                    stream_structure |= 0x02;
                }

                let combined = (stream_structure << 4) | (format_ordering & 0x0F);
                combined.encode(buf)?;

                base_channel_count.encode(buf)?;

                self.encode_channel_structure_v1(buf)?;

                if object_structured {
                    Self::encode_object_structure_v1(
                        buf,
                        self.channel_structure.as_ref(),
                        base_channel_count,
                    )?;
                }
            }
        }

        Ok(ChnlExt { version })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chnl_v0_explicit_positions() {
        let chnl = Chnl {
            channel_structure: Some(ChannelStructure::ExplicitPositions {
                positions: vec![
                    SpeakerPosition::Standard(AudioChannelPosition::FrontLeft),
                    SpeakerPosition::Standard(AudioChannelPosition::FrontRight),
                ],
            }),
            object_count: None,
            format_ordering: None,
            base_channel_count: None,
        };

        let mut buf = Vec::new();
        chnl.encode(&mut buf).unwrap();

        let decoded = Chnl::decode(&mut &buf[..]).unwrap();
        assert_eq!(chnl, decoded);
    }

    #[test]
    fn test_chnl_v0_defined_layout() {
        let chnl = Chnl {
            channel_structure: Some(ChannelStructure::DefinedLayout {
                layout: 2,
                omitted_channels_map: Some(0),
                channel_order_definition: None,
            }),
            object_count: None,
            format_ordering: None,
            base_channel_count: None,
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
            format_ordering: None,
            base_channel_count: None,
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

    #[test]
    fn test_chnl_v1_explicit_positions() {
        let chnl = Chnl {
            channel_structure: Some(ChannelStructure::ExplicitPositions {
                positions: vec![
                    SpeakerPosition::Standard(AudioChannelPosition::FrontLeft),
                    SpeakerPosition::Standard(AudioChannelPosition::FrontRight),
                    SpeakerPosition::Standard(AudioChannelPosition::FrontCenter),
                ],
            }),
            object_count: None,
            format_ordering: Some(1),
            base_channel_count: Some(3),
        };

        let mut buf = Vec::new();
        chnl.encode(&mut buf).unwrap();

        let decoded = Chnl::decode(&mut &buf[..]).unwrap();
        assert_eq!(chnl, decoded);
    }

    #[test]
    fn test_chnl_v1_defined_layout_with_omitted() {
        let chnl = Chnl {
            channel_structure: Some(ChannelStructure::DefinedLayout {
                layout: 6,
                omitted_channels_map: Some(0x20), // Omit bit 5
                channel_order_definition: Some(0),
            }),
            object_count: None,
            format_ordering: Some(1),
            base_channel_count: Some(6),
        };

        let mut buf = Vec::new();
        chnl.encode(&mut buf).unwrap();

        let decoded = Chnl::decode(&mut &buf[..]).unwrap();
        assert_eq!(chnl, decoded);
    }

    #[test]
    fn test_chnl_v1_defined_layout_without_omitted() {
        let chnl = Chnl {
            channel_structure: Some(ChannelStructure::DefinedLayout {
                layout: 2,
                omitted_channels_map: None,
                channel_order_definition: Some(0),
            }),
            object_count: None,
            format_ordering: Some(1),
            base_channel_count: Some(2),
        };

        let mut buf = Vec::new();
        chnl.encode(&mut buf).unwrap();

        let decoded = Chnl::decode(&mut &buf[..]).unwrap();
        assert_eq!(chnl, decoded);
    }

    #[test]
    fn test_chnl_unpositioned_audio() {
        let chnl = Chnl {
            channel_structure: Some(ChannelStructure::ExplicitPositions {
                positions: vec![
                    SpeakerPosition::Standard(AudioChannelPosition::Unknown),
                    SpeakerPosition::Standard(AudioChannelPosition::Unknown),
                ],
            }),
            object_count: None,
            format_ordering: None,
            base_channel_count: None,
        };

        let mut buf = Vec::new();
        chnl.encode(&mut buf).unwrap();

        let decoded = Chnl::decode(&mut &buf[..]).unwrap();
        assert_eq!(chnl, decoded);
    }

    #[test]
    fn test_chnl_v1_with_objects() {
        let chnl = Chnl {
            channel_structure: Some(ChannelStructure::ExplicitPositions {
                positions: vec![
                    SpeakerPosition::Standard(AudioChannelPosition::FrontLeft),
                    SpeakerPosition::Standard(AudioChannelPosition::FrontRight),
                ],
            }),
            object_count: None,
            format_ordering: Some(1),
            base_channel_count: Some(5),
        };

        let mut buf = Vec::new();
        chnl.encode(&mut buf).unwrap();

        let decoded = Chnl::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded.object_count, Some(3));
    }

    #[test]
    fn test_chnl_explicit_speaker_positions() {
        let chnl = Chnl {
            channel_structure: Some(ChannelStructure::ExplicitPositions {
                positions: vec![
                    SpeakerPosition::Standard(AudioChannelPosition::FrontLeft),
                    SpeakerPosition::Explicit(ExplicitSpeakerPosition {
                        azimuth: 30,
                        elevation: 0,
                    }),
                    SpeakerPosition::Standard(AudioChannelPosition::FrontRight),
                ],
            }),
            object_count: None,
            format_ordering: None,
            base_channel_count: None,
        };

        let mut buf = Vec::new();
        chnl.encode(&mut buf).unwrap();

        let decoded = Chnl::decode(&mut &buf[..]).unwrap();
        assert_eq!(chnl, decoded);
    }
}
