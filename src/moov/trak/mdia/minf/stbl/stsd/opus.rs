use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Opus {
    pub audio: Audio,
    pub dops: Dops,
    pub btrt: Option<Btrt>,
}

impl Atom for Opus {
    const KIND: FourCC = FourCC::new(b"Opus");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let audio = Audio::decode(buf)?;

        let mut dops = None;
        let mut btrt = None;

        // Find d0ps in mp4a or wave
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Dops(atom) => dops = atom.into(),
                Any::Btrt(atom) => btrt = atom.into(),
                unknown => Self::decode_unknown(&unknown)?,
            }
        }
        skip_trailing_padding(buf);

        Ok(Self {
            audio,
            dops: dops.ok_or(Error::MissingBox(Dops::KIND))?,
            btrt,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.audio.encode(buf)?;
        self.dops.encode(buf)?;
        self.btrt.encode(buf)?;
        Ok(())
    }
}

/*
    class ChannelMappingTable (unsigned int(8) OutputChannelCount){
        unsigned int(8) StreamCount;
        unsigned int(8) CoupledCount;
        unsigned int(8 * OutputChannelCount) ChannelMapping;
    }

    aligned(8) class OpusSpecificBox extends Box('dOps'){
        unsigned int(8) Version;
        unsigned int(8) OutputChannelCount;
        unsigned int(16) PreSkip;
        unsigned int(32) InputSampleRate;
        signed int(16) OutputGain;
        unsigned int(8) ChannelMappingFamily;
        if (ChannelMappingFamily != 0) {
            ChannelMappingTable(OutputChannelCount);
        }
    }
*/

// Opus specific data
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dops {
    pub output_channel_count: u8,
    pub pre_skip: u16,
    pub input_sample_rate: u32,
    pub output_gain: i16,
    /// The channel mapping family and table, or `None` for family 0 (mono or stereo, no table).
    pub channel_mapping: Option<OpusChannelMapping>,
}

/// A nonzero channel mapping family and its table, laid out as RFC 7845 §5.1.1 describes.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpusChannelMapping {
    /// The channel mapping family; never 0, which has no table.
    pub family: u8,
    /// The number of Opus streams in each packet.
    pub stream_count: u8,
    /// How many of those streams are coupled (stereo); they come first.
    pub coupled_count: u8,
    /// The decoded channel feeding each output channel, or 255 for silence.
    /// It has one entry per output channel.
    pub channel_mapping: Vec<u8>,
}

impl OpusChannelMapping {
    // The checks RFC 7845 §5.1.1 applies to every family, so a table read from
    // a file is refused rather than trusted.
    fn validate(&self, output_channel_count: u8) -> Result<()> {
        if self.family == 0 {
            return Err(Error::InvalidCombination(
                "Opus channel mapping family 0 has no table",
            ));
        }
        if output_channel_count == 0 {
            return Err(Error::InvalidCombination(
                "Opus output channel count must not be zero",
            ));
        }
        if self.channel_mapping.len() != output_channel_count as usize {
            return Err(Error::InvalidCombination(
                "Opus channel mapping needs one entry per output channel",
            ));
        }
        if self.stream_count == 0 {
            return Err(Error::InvalidCombination(
                "Opus stream count must not be zero",
            ));
        }
        if self.coupled_count > self.stream_count {
            return Err(Error::InvalidCombination(
                "Opus coupled count exceeds the stream count",
            ));
        }

        let decoded = self.stream_count as u16 + self.coupled_count as u16;
        if decoded > 255 {
            return Err(Error::InvalidCombination(
                "Opus streams decode to more than 255 channels",
            ));
        }
        if self
            .channel_mapping
            .iter()
            .any(|&index| index != 255 && index as u16 >= decoded)
        {
            return Err(Error::InvalidCombination(
                "Opus channel mapping names a channel past the decoded ones",
            ));
        }

        Ok(())
    }
}

impl Atom for Dops {
    const KIND: FourCC = FourCC::new(b"dOps");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let version = u8::decode(buf)?;
        if version != 0 {
            return Err(Error::UnknownVersion(version));
        }

        let output_channel_count = u8::decode(buf)?;
        let pre_skip = u16::decode(buf)?;
        let input_sample_rate = u32::decode(buf)?;
        let output_gain = i16::decode(buf)?;

        let family = u8::decode(buf)?;
        let channel_mapping = match family {
            0 => None,
            family => {
                let mapping = OpusChannelMapping {
                    family,
                    stream_count: u8::decode(buf)?,
                    coupled_count: u8::decode(buf)?,
                    channel_mapping: Vec::decode_exact(buf, output_channel_count as usize)?,
                };
                mapping.validate(output_channel_count)?;
                Some(mapping)
            }
        };

        Ok(Self {
            output_channel_count,
            pre_skip,
            input_sample_rate,
            output_gain,
            channel_mapping,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        if let Some(mapping) = &self.channel_mapping {
            mapping.validate(self.output_channel_count)?;
        }

        (0u8).encode(buf)?;
        self.output_channel_count.encode(buf)?;
        self.pre_skip.encode(buf)?;
        self.input_sample_rate.encode(buf)?;
        self.output_gain.encode(buf)?;

        match &self.channel_mapping {
            None => (0u8).encode(buf)?,
            Some(mapping) => {
                mapping.family.encode(buf)?;
                mapping.stream_count.encode(buf)?;
                mapping.coupled_count.encode(buf)?;
                mapping.channel_mapping.encode(buf)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A stereo dOps with family 0 and no table.
    const ENCODED_DOPS_STEREO: &[u8] = &[
        0x00, 0x00, 0x00, 0x13, b'd', b'O', b'p', b's', 0x00, 0x02, 0x01, 0x38, 0x00, 0x00, 0xbb,
        0x80, 0xff, 0xfe, 0x00,
    ];

    // A 5.1 dOps with family 1 and the Vorbis order table libopus uses.
    const ENCODED_DOPS_SURROUND: &[u8] = &[
        0x00, 0x00, 0x00, 0x1b, b'd', b'O', b'p', b's', 0x00, 0x06, 0x01, 0x38, 0x00, 0x00, 0xbb,
        0x80, 0x00, 0x00, 0x01, 0x04, 0x02, 0x00, 0x04, 0x01, 0x02, 0x03, 0x05,
    ];

    fn surround() -> Dops {
        Dops {
            output_channel_count: 6,
            pre_skip: 312,
            input_sample_rate: 48000,
            output_gain: 0,
            channel_mapping: Some(OpusChannelMapping {
                family: 1,
                stream_count: 4,
                coupled_count: 2,
                channel_mapping: vec![0, 4, 1, 2, 3, 5],
            }),
        }
    }

    fn decode(bytes: &[u8]) -> Result<Dops> {
        Dops::decode(&mut std::io::Cursor::new(bytes))
    }

    fn encode(dops: &Dops) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        dops.encode(&mut buf)?;
        Ok(buf)
    }

    #[test]
    fn test_dops_family_0() {
        let dops = Dops {
            output_channel_count: 2,
            pre_skip: 312,
            input_sample_rate: 48000,
            output_gain: -2,
            channel_mapping: None,
        };

        assert_eq!(decode(ENCODED_DOPS_STEREO).unwrap(), dops);
        assert_eq!(encode(&dops).unwrap(), ENCODED_DOPS_STEREO);
    }

    #[test]
    fn test_dops_family_1() {
        let dops = surround();

        assert_eq!(decode(ENCODED_DOPS_SURROUND).unwrap(), dops);
        assert_eq!(encode(&dops).unwrap(), ENCODED_DOPS_SURROUND);
    }

    #[test]
    fn test_dops_any_family() {
        for family in [2, 3, 255] {
            let mut dops = surround();
            dops.channel_mapping.as_mut().unwrap().family = family;

            let encoded = encode(&dops).unwrap();
            assert_eq!(decode(&encoded).unwrap(), dops);
        }
    }

    #[test]
    fn test_dops_silent_channel() {
        let mut dops = surround();
        dops.channel_mapping.as_mut().unwrap().channel_mapping[5] = 255;

        let encoded = encode(&dops).unwrap();
        assert_eq!(decode(&encoded).unwrap(), dops);
    }

    #[test]
    fn test_dops_truncated_table() {
        // Drop the last table entry, leaving five for six output channels.
        let mut bytes = ENCODED_DOPS_SURROUND[..ENCODED_DOPS_SURROUND.len() - 1].to_vec();
        bytes[3] -= 1;

        assert!(decode(&bytes).is_err());
    }

    #[test]
    fn test_dops_rejects_invalid_table() {
        let invalid: [fn(&mut OpusChannelMapping); 4] = [
            |mapping| mapping.stream_count = 0,
            |mapping| mapping.coupled_count = mapping.stream_count + 1,
            |mapping| mapping.channel_mapping[0] = 6,
            |mapping| {
                mapping.stream_count = 200;
                mapping.coupled_count = 100;
            },
        ];

        for corrupt in invalid {
            let mut dops = surround();
            corrupt(dops.channel_mapping.as_mut().unwrap());
            assert!(matches!(encode(&dops), Err(Error::InvalidCombination(_))));

            // Write the same table past the encoder's check to prove decode refuses it too.
            let mapping = dops.channel_mapping.as_ref().unwrap();
            let mut bytes = ENCODED_DOPS_SURROUND.to_vec();
            bytes[19] = mapping.stream_count;
            bytes[20] = mapping.coupled_count;
            bytes[21..].copy_from_slice(&mapping.channel_mapping);
            assert!(matches!(decode(&bytes), Err(Error::InvalidCombination(_))));
        }
    }

    #[test]
    fn test_dops_rejects_family_0_table() {
        let mut dops = surround();
        dops.channel_mapping.as_mut().unwrap().family = 0;

        assert!(matches!(encode(&dops), Err(Error::InvalidCombination(_))));
    }

    #[test]
    fn test_dops_rejects_table_length_mismatch() {
        let mut dops = surround();
        dops.output_channel_count = 5;

        assert!(matches!(encode(&dops), Err(Error::InvalidCombination(_))));
    }
}
