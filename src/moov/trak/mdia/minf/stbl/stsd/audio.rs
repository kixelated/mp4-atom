use crate::coding::{Decode, Encode};
use crate::{Buf, BufMut, Error, Result};

/// Version-specific fields in an audio sample entry.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioVersion {
    /// ISO/QuickTime version 0 sound sample description.
    #[default]
    V0,

    /// QuickTime version 1 sound sample description.
    V1 {
        samples_per_packet: u32,
        bytes_per_packet: u32,
        bytes_per_frame: u32,
        bytes_per_sample: u32,
    },

    /// QuickTime version 2 sound sample description.
    V2 {
        size_of_struct_only: u32,
        always_7f000000: u32,
        bits_per_channel: u32,
        format_specific_flags: u32,
        bytes_per_audio_packet: u32,
        lpcm_frames_per_audio_packet: u32,
    },
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Audio {
    pub data_reference_index: u16,
    pub version: AudioVersion,
    pub channel_count: u32,
    /// Legacy sample-size field.
    ///
    /// For version 2 entries, the authoritative value is the
    /// `bits_per_channel` field in [`AudioVersion::V2`].
    pub sample_size: u16,
    /// Sample rate in Hz.
    ///
    /// Version 0 and 1 entries store this as unsigned 16.16 fixed-point,
    /// while version 2 entries store it as an IEEE-754 `f64`.
    pub sample_rate: f64,
}

// Compare the encoded representation of the rate so Audio remains Eq even for
// unusual version 2 values such as NaN, and so re-encoding preserves the bits.
impl PartialEq for Audio {
    fn eq(&self, other: &Self) -> bool {
        self.data_reference_index == other.data_reference_index
            && self.version == other.version
            && self.channel_count == other.channel_count
            && self.sample_size == other.sample_size
            && self.sample_rate.to_bits() == other.sample_rate.to_bits()
    }
}

impl Eq for Audio {}

impl Audio {
    fn decode_fixed_16_16(raw: u32) -> f64 {
        f64::from(raw) / 65536.0
    }

    fn encode_fixed_16_16(value: f64) -> Result<u32> {
        let scaled = value * 65536.0;
        if !scaled.is_finite() || scaled < 0.0 || scaled > f64::from(u32::MAX) {
            return Err(Error::Unsupported(
                "sample rate does not fit unsigned 16.16 fixed-point",
            ));
        }

        Ok(scaled.round() as u32)
    }
}

impl Encode for Audio {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        0u32.encode(buf)?; // reserved
        0u16.encode(buf)?; // reserved
        self.data_reference_index.encode(buf)?;

        let (version, channel_count, sample_size, compression_id, sample_rate) = match self.version
        {
            AudioVersion::V0 => (
                0u16,
                u16::try_from(self.channel_count)
                    .map_err(|_| Error::Unsupported("version 0 channel count exceeds u16"))?,
                self.sample_size,
                0i16,
                Self::encode_fixed_16_16(self.sample_rate)?,
            ),
            AudioVersion::V1 { .. } => (
                1u16,
                u16::try_from(self.channel_count)
                    .map_err(|_| Error::Unsupported("version 1 channel count exceeds u16"))?,
                self.sample_size,
                -1i16,
                Self::encode_fixed_16_16(self.sample_rate)?,
            ),
            // QuickTime mandates these placeholders for version 2. The
            // authoritative values are written in the extension below.
            AudioVersion::V2 { .. } => (2u16, 3, 16, -2i16, 0x0001_0000),
        };

        version.encode(buf)?;
        0u16.encode(buf)?; // revision level
        0u32.encode(buf)?; // vendor
        channel_count.encode(buf)?;
        sample_size.encode(buf)?;
        compression_id.encode(buf)?;
        0u16.encode(buf)?; // packet size
        sample_rate.encode(buf)?;

        match self.version {
            AudioVersion::V0 => {}
            AudioVersion::V1 {
                samples_per_packet,
                bytes_per_packet,
                bytes_per_frame,
                bytes_per_sample,
            } => {
                samples_per_packet.encode(buf)?;
                bytes_per_packet.encode(buf)?;
                bytes_per_frame.encode(buf)?;
                bytes_per_sample.encode(buf)?;
            }
            AudioVersion::V2 {
                size_of_struct_only,
                always_7f000000,
                bits_per_channel,
                format_specific_flags,
                bytes_per_audio_packet,
                lpcm_frames_per_audio_packet,
            } => {
                size_of_struct_only.encode(buf)?;
                self.sample_rate.to_bits().encode(buf)?;
                self.channel_count.encode(buf)?;
                always_7f000000.encode(buf)?;
                bits_per_channel.encode(buf)?;
                format_specific_flags.encode(buf)?;
                bytes_per_audio_packet.encode(buf)?;
                lpcm_frames_per_audio_packet.encode(buf)?;
            }
        }

        Ok(())
    }
}

impl Decode for Audio {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        u32::decode(buf)?; // reserved
        u16::decode(buf)?; // reserved
        let data_reference_index = u16::decode(buf)?;
        let version = u16::decode(buf)?;
        u16::decode(buf)?; // revision level
        u32::decode(buf)?; // vendor
        let legacy_channel_count = u16::decode(buf)?;
        let sample_size = u16::decode(buf)?;
        i16::decode(buf)?; // compression ID
        u16::decode(buf)?; // packet size
        let legacy_sample_rate = u32::decode(buf)?;

        let (version, channel_count, sample_rate) = match version {
            0 => (
                AudioVersion::V0,
                u32::from(legacy_channel_count),
                Self::decode_fixed_16_16(legacy_sample_rate),
            ),
            1 => {
                let version = AudioVersion::V1 {
                    samples_per_packet: u32::decode(buf)?,
                    bytes_per_packet: u32::decode(buf)?,
                    bytes_per_frame: u32::decode(buf)?,
                    bytes_per_sample: u32::decode(buf)?,
                };
                (
                    version,
                    u32::from(legacy_channel_count),
                    Self::decode_fixed_16_16(legacy_sample_rate),
                )
            }
            2 => {
                let size_of_struct_only = u32::decode(buf)?;
                let sample_rate = f64::from_bits(u64::decode(buf)?);
                let channel_count = u32::decode(buf)?;
                let version = AudioVersion::V2 {
                    size_of_struct_only,
                    always_7f000000: u32::decode(buf)?,
                    bits_per_channel: u32::decode(buf)?,
                    format_specific_flags: u32::decode(buf)?,
                    bytes_per_audio_packet: u32::decode(buf)?,
                    lpcm_frames_per_audio_packet: u32::decode(buf)?,
                };
                (version, channel_count, sample_rate)
            }
            n => return Err(Error::UnknownQuicktimeVersion(n)),
        };

        Ok(Self {
            data_reference_index,
            version,
            channel_count,
            sample_size,
            sample_rate,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VERSION_2: &[u8] = &[
        0, 0, 0, 0, 0, 0, // reserved
        0, 1, // data reference index
        0, 2, // version
        0, 0, // revision level
        0, 0, 0, 0, // vendor
        0, 3, // placeholder channel count
        0, 16, // placeholder sample size
        0xff, 0xfe, // compression ID
        0, 0, // packet size
        0, 1, 0, 0, // placeholder sample rate (1.0)
        0, 0, 0, 72, // size of struct only
        0x40, 0xf7, 0x70, 0, 0, 0, 0, 0, // sample rate (96000.0)
        0, 0, 0, 2, // channel count
        0x7f, 0, 0, 0, // always 0x7f000000
        0, 0, 0, 24, // bits per channel
        0, 0, 0, 12, // signed integer, packed
        0, 0, 0, 6, // bytes per audio packet
        0, 0, 0, 1, // LPCM frames per audio packet
    ];

    #[test]
    fn version_2_uses_authoritative_rate_and_channel_count() {
        let mut buf = VERSION_2;
        let audio = Audio::decode(&mut buf).unwrap();

        assert_eq!(audio.channel_count, 2);
        assert_eq!(audio.sample_rate, 96000.0);
        assert!(matches!(audio.version, AudioVersion::V2 { .. }));
    }

    #[test]
    fn version_2_roundtrips_without_becoming_version_0() {
        let mut buf = VERSION_2;
        let audio = Audio::decode(&mut buf).unwrap();
        let mut encoded = Vec::new();
        audio.encode(&mut encoded).unwrap();

        assert_eq!(encoded, VERSION_2);
    }

    #[test]
    fn version_0_rejects_a_rate_that_needs_version_2() {
        let audio = Audio {
            data_reference_index: 1,
            version: AudioVersion::V0,
            channel_count: 2,
            sample_size: 24,
            sample_rate: 96000.0,
        };

        assert!(matches!(
            audio.encode(&mut Vec::new()),
            Err(Error::Unsupported(_))
        ));
    }
}
