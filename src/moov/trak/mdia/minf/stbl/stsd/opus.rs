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

        let channel_mapping_family = u8::decode(buf)?;
        if channel_mapping_family != 0 {
            return Err(Error::Unsupported("OPUS channel mapping"));
        }

        Ok(Self {
            output_channel_count,
            pre_skip,
            input_sample_rate,
            output_gain,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        (0u8).encode(buf)?;
        self.output_channel_count.encode(buf)?;
        self.pre_skip.encode(buf)?;
        self.input_sample_rate.encode(buf)?;
        self.output_gain.encode(buf)?;
        (0u8).encode(buf)?;

        Ok(())
    }
}
