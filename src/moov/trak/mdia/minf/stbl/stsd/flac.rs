use crate::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Flac {
    pub audio: Audio,
    pub dfla: Dfla,
    #[cfg(feature = "fault-tolerant")]
    pub unexpected: Vec<Any>,
}

impl Atom for Flac {
    const KIND: FourCC = FourCC::new(b"fLaC");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let audio = Audio::decode(buf)?;

        let mut dfla = None;
        #[cfg(feature = "fault-tolerant")]
        let mut unexpected = Vec::new();

        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Dfla(atom) => dfla = atom.into(),
                _ => {
                    tracing::warn!("unknown atom: {:?}", atom);
                    #[cfg(feature = "fault-tolerant")]
                    unexpected.push(atom);
                }
            }
        }

        Ok(Self {
            audio,
            dfla: dfla.ok_or(Error::MissingBox(Dfla::KIND))?,
            #[cfg(feature = "fault-tolerant")]
            unexpected,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.audio.encode(buf)?;
        self.dfla.encode(buf)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FlacMetadataBlock {
    StreamInfo {
        minimum_block_size: u16,
        maximum_block_size: u16,
        minimum_frame_size: u24,
        maximum_frame_size: u24,
        sample_rate: u32,
        num_channels_minus_one: u8,
        bits_per_sample_minus_one: u8,
        number_of_interchannel_samples: u64,
        md5_checksum: Vec<u8>,
    },
    Padding,
    Application,
    SeekTable,
    VorbisComment {
        vendor_string: String,
        comments: Vec<String>,
    },
    CueSheet,
    Picture,
    Reserved,
    Forbidden,
}

impl FlacMetadataBlock {
    fn encode_initial_byte<B: BufMut>(buf: &mut B, block_type: u8, is_last: bool) -> Result<()> {
        match is_last {
            true => (0x80 | block_type).encode(buf),
            false => block_type.encode(buf),
        }
    }
    fn encode<B: BufMut>(&self, buf: &mut B, is_last: bool) -> Result<()> {
        match self {
            FlacMetadataBlock::StreamInfo {
                minimum_block_size,
                maximum_block_size,
                minimum_frame_size,
                maximum_frame_size,
                sample_rate,
                num_channels_minus_one,
                bits_per_sample_minus_one,
                number_of_interchannel_samples,
                md5_checksum,
            } => {
                Self::encode_initial_byte(buf, 0u8, is_last)?;
                // Add a u24 length placeholder
                u24::from([0u8, 0u8, 0u8]).encode(buf)?;
                let length_position = buf.len();
                minimum_block_size.encode(buf)?;
                maximum_block_size.encode(buf)?;
                minimum_frame_size.encode(buf)?;
                maximum_frame_size.encode(buf)?;
                (((*sample_rate as u64) << 44)
                    | ((*num_channels_minus_one as u64) << 41)
                    | ((*bits_per_sample_minus_one as u64) << 36)
                    | number_of_interchannel_samples)
                    .encode(buf)?;
                if md5_checksum.len() != 16 {
                    return Err(Error::MissingContent(
                        "StreamInfo.md5_checksum must be 16 bytes",
                    ));
                }
                md5_checksum.encode(buf)?;
                let length: u24 = ((buf.len() - length_position) as u32)
                    .try_into()
                    .map_err(|_| Error::TooLarge(Dfla::KIND))?;
                buf.set_slice(length_position - 3, &length.to_be_bytes());
            }
            FlacMetadataBlock::Padding => { /* cannot write this yet */ }
            FlacMetadataBlock::Application => { /* cannot write this yet */ }
            FlacMetadataBlock::SeekTable => { /* cannot write this yet */ }
            FlacMetadataBlock::VorbisComment {
                vendor_string,
                comments,
            } => {
                Self::encode_initial_byte(buf, 4u8, is_last)?;

                // Add a u24 length placeholder
                u24::from([0u8, 0u8, 0u8]).encode(buf)?;
                let length_position = buf.len();
                let vendor_string_bytes = vendor_string.as_bytes();
                let vendor_string_len: u32 = vendor_string_bytes.len() as u32;
                vendor_string_len.to_le_bytes().encode(buf)?;
                vendor_string_bytes.encode(buf)?;
                let number_of_comments: u32 = comments.len() as u32;
                number_of_comments.to_le_bytes().encode(buf)?;
                for comment in comments {
                    let comment_bytes = comment.as_bytes();
                    let comment_len: u32 = comment_bytes.len() as u32;
                    comment_len.to_le_bytes().encode(buf)?;
                    comment_bytes.encode(buf)?;
                }
                let length: u24 = ((buf.len() - length_position) as u32)
                    .try_into()
                    .map_err(|_| Error::TooLarge(Dfla::KIND))?;
                buf.set_slice(length_position - 3, &length.to_be_bytes());
            }
            FlacMetadataBlock::CueSheet => { /* cannot write this yet */ }
            FlacMetadataBlock::Picture => { /* cannot write this yet */ }
            FlacMetadataBlock::Reserved => { /* No way to write this */ }
            FlacMetadataBlock::Forbidden => { /* No way to write this */ }
        }
        Ok(())
    }
}

// FLAC specific data
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dfla {
    pub blocks: Vec<FlacMetadataBlock>,
}

fn parse_stream_info(arr: &[u8]) -> Result<FlacMetadataBlock> {
    // See RFC 9639 Section 8.2
    let buf = &mut std::io::Cursor::new(arr);
    let minimum_block_size = u16::decode(buf)?;
    let maximum_block_size = u16::decode(buf)?;
    let minimum_frame_size = u24::decode(buf)?;
    let maximum_frame_size = u24::decode(buf)?;
    let temp64 = u64::decode(buf)?;
    let sample_rate: u32 = (temp64 >> 44) as u32; // 20 bits
    let num_channels_minus_one: u8 = ((temp64 >> 41) & 0x07) as u8; // 3 bits
    let bits_per_sample_minus_one: u8 = ((temp64 >> 36) & 0x1F) as u8; // 5 bits
    let number_of_interchannel_samples: u64 = temp64 & 0x0000_000F_FFFF_FFFF; // 36 bits
    let md5_checksum: Vec<u8> = Vec::decode_exact(buf, 16)?;
    Ok(FlacMetadataBlock::StreamInfo {
        minimum_block_size,
        maximum_block_size,
        minimum_frame_size,
        maximum_frame_size,
        sample_rate,
        num_channels_minus_one,
        bits_per_sample_minus_one,
        number_of_interchannel_samples,
        md5_checksum,
    })
}

fn parse_vorbis_comment(arr: &[u8]) -> Result<FlacMetadataBlock> {
    // See RFC 9639 Section 8.6, and D.2.5 for an example
    // Vorbis comments in FLAC use little endian, for Vorbis compatibility
    let buf = &mut std::io::Cursor::new(arr);
    let vendor_string_length = u32::from_le_bytes(<[u8; 4]>::decode(buf)?) as usize;
    let vendor_string_bytes: Vec<u8> = Vec::decode_exact(buf, vendor_string_length)?;
    let vendor_string = String::from_utf8_lossy(&vendor_string_bytes)
        .trim_end_matches('\0')
        .to_string();
    let number_of_fields = u32::from_le_bytes(<[u8; 4]>::decode(buf)?) as usize;
    let mut comments = Vec::with_capacity(number_of_fields);
    for _ in 0..number_of_fields {
        let field_length = u32::from_le_bytes(<[u8; 4]>::decode(buf)?) as usize;
        let field_bytes: Vec<u8> = Vec::decode_exact(buf, field_length)?;
        let comment = String::from_utf8_lossy(&field_bytes)
            .trim_end_matches('\0')
            .to_string();
        comments.push(comment);
    }
    Ok(FlacMetadataBlock::VorbisComment {
        vendor_string,
        comments,
    })
}

impl AtomExt for Dfla {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"dfLa");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let mut is_last_block = false;
        let mut metadata_blocks = Vec::new();
        while buf.has_remaining() && !is_last_block {
            let initial_bytes = u32::decode(buf)?;
            is_last_block = (initial_bytes & 0x80_00_00_00) == 0x80_00_00_00;
            let block_type = ((initial_bytes >> 24) & 0x7f) as u8;
            let length = initial_bytes & 0x00_FF_FF_FF;
            let block_data: Vec<u8> = Vec::decode_exact(buf, length as usize)?;
            let metadata_block = match block_type {
                0 => parse_stream_info(&block_data)?,
                1 => FlacMetadataBlock::Padding,
                2 => FlacMetadataBlock::Application,
                3 => FlacMetadataBlock::SeekTable,
                4 => parse_vorbis_comment(&block_data)?,
                5 => FlacMetadataBlock::CueSheet,
                6 => FlacMetadataBlock::Picture,
                7..=126 => FlacMetadataBlock::Reserved,
                127 => FlacMetadataBlock::Forbidden,
                _ => unreachable!("FLAC Metadata Block type is only 7 bits"),
            };
            metadata_blocks.push(metadata_block);
        }
        Ok(Dfla {
            blocks: metadata_blocks,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        if self.blocks.is_empty() {
            return Err(Error::MissingContent("Streaminfo"));
        }
        for (i, metadata_block) in self.blocks.iter().enumerate() {
            let is_last = i + 1 == self.blocks.len();
            metadata_block.encode(buf, is_last)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Streaminfo metadata block only
    const ENCODED_DFLA: &[u8] = &[
        0x00, 0x00, 0x00, 0x32, 0x64, 0x66, 0x4c, 0x61, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00,
        0x22, 0x12, 0x00, 0x12, 0x00, 0x00, 0x00, 0x10, 0x00, 0x23, 0x8e, 0x0a, 0xc4, 0x43, 0x70,
        0x00, 0x01, 0xd8, 0x00, 0x75, 0x30, 0x88, 0x11, 0x2d, 0xd5, 0x7a, 0x13, 0xe7, 0xf7, 0x22,
        0xd0, 0xee, 0x56, 0xae, 0xa3,
    ];

    #[test]
    fn test_dfla_decode() {
        let buf: &mut std::io::Cursor<&[u8]> = &mut std::io::Cursor::new(ENCODED_DFLA);

        let dfla = Dfla::decode(buf).expect("failed to decode dfLa");

        assert_eq!(
            dfla,
            Dfla {
                blocks: vec![FlacMetadataBlock::StreamInfo {
                    minimum_block_size: 4608,
                    maximum_block_size: 4608,
                    minimum_frame_size: 16u32.try_into().expect("should fit in u24"),
                    maximum_frame_size: 9102u32.try_into().expect("should fit in u24"),
                    sample_rate: 44100,
                    num_channels_minus_one: 1,
                    bits_per_sample_minus_one: 23,
                    number_of_interchannel_samples: 120832,
                    md5_checksum: vec![
                        117, 48, 136, 17, 45, 213, 122, 19, 231, 247, 34, 208, 238, 86, 174, 163
                    ]
                },]
            }
        );
    }

    #[test]
    fn test_dfla_encode() {
        let dfla = Dfla {
            blocks: vec![FlacMetadataBlock::StreamInfo {
                minimum_block_size: 4608,
                maximum_block_size: 4608,
                minimum_frame_size: 16u32.try_into().expect("should fit in u24"),
                maximum_frame_size: 9102u32.try_into().expect("should fit in u24"),
                sample_rate: 44100,
                num_channels_minus_one: 1,
                bits_per_sample_minus_one: 23,
                number_of_interchannel_samples: 120832,
                md5_checksum: vec![
                    117, 48, 136, 17, 45, 213, 122, 19, 231, 247, 34, 208, 238, 86, 174, 163,
                ],
            }],
        };

        let mut buf = Vec::new();
        dfla.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_DFLA);
    }

    // Streaminfo metadata block plus Vorbis Comment metadata block
    const ENCODED_DFLA_2: &[u8] = &[
        0x00, 0x00, 0x00, 0x7c, 0x64, 0x66, 0x4c, 0x61, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x22, 0x12, 0x00, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0a, 0xc4, 0x40, 0x70,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x84, 0x00, 0x00, 0x46, 0x20, 0x00, 0x00, 0x00, 0x72, 0x65,
        0x66, 0x65, 0x72, 0x65, 0x6e, 0x63, 0x65, 0x20, 0x6c, 0x69, 0x62, 0x46, 0x4c, 0x41, 0x43,
        0x20, 0x31, 0x2e, 0x34, 0x2e, 0x33, 0x20, 0x32, 0x30, 0x32, 0x33, 0x30, 0x36, 0x32, 0x33,
        0x01, 0x00, 0x00, 0x00, 0x1a, 0x00, 0x00, 0x00, 0x44, 0x45, 0x53, 0x43, 0x52, 0x49, 0x50,
        0x54, 0x49, 0x4f, 0x4e, 0x3d, 0x61, 0x75, 0x64, 0x69, 0x6f, 0x74, 0x65, 0x73, 0x74, 0x20,
        0x77, 0x61, 0x76, 0x65,
    ];

    #[test]
    fn test_dfla2_decode() {
        let buf: &mut std::io::Cursor<&[u8]> = &mut std::io::Cursor::new(ENCODED_DFLA_2);

        let dfla = Dfla::decode(buf).expect("failed to decode dfLa");

        assert_eq!(
            dfla,
            Dfla {
                blocks: vec![
                    FlacMetadataBlock::StreamInfo {
                        minimum_block_size: 4608,
                        maximum_block_size: 4608,
                        minimum_frame_size: 0u32.try_into().expect("should fit in u24"),
                        maximum_frame_size: 0u32.try_into().expect("should fit in u24"),
                        sample_rate: 44100,
                        num_channels_minus_one: 0,
                        bits_per_sample_minus_one: 7,
                        number_of_interchannel_samples: 0,
                        md5_checksum: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
                    },
                    FlacMetadataBlock::VorbisComment {
                        vendor_string: "reference libFLAC 1.4.3 20230623".into(),
                        comments: vec!["DESCRIPTION=audiotest wave".into()],
                    },
                ]
            }
        );
    }

    #[test]
    fn test_dfla2_encode() {
        let dfla = Dfla {
            blocks: vec![
                FlacMetadataBlock::StreamInfo {
                    minimum_block_size: 4608,
                    maximum_block_size: 4608,
                    minimum_frame_size: 0u32.try_into().expect("should fit in u24"),
                    maximum_frame_size: 0u32.try_into().expect("should fit in u24"),
                    sample_rate: 44100,
                    num_channels_minus_one: 0,
                    bits_per_sample_minus_one: 7,
                    number_of_interchannel_samples: 0,
                    md5_checksum: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                },
                FlacMetadataBlock::VorbisComment {
                    vendor_string: "reference libFLAC 1.4.3 20230623".into(),
                    comments: vec!["DESCRIPTION=audiotest wave".into()],
                },
            ],
        };

        let mut buf = Vec::new();
        dfla.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_DFLA_2);
    }
}
