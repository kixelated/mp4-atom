use crate::coding::{Decode, Encode};
use crate::{Buf, BufMut, Error, FixedPoint, Result};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Audio {
    pub data_reference_index: u16,
    pub channel_count: u16,
    pub sample_size: u16,
    pub sample_rate: FixedPoint<u16>,
}

/// The QuickTime **version-2** sound sample description fields (QTFF).
///
/// A version-2 `AudioSampleEntry` (which `lpcm` mandates, and which carries
/// rates/channel-counts the legacy fields cannot represent) sets the base
/// [`Audio`] `channel_count` / `sample_size` / `sample_rate` to spec-mandated
/// placeholders; the real values live here, alongside the CoreAudio LPCM
/// `format_flags` that name the storage format (float vs int, endianness).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SoundV2 {
    /// `audioSampleRate` — the real rate (the on-disk `f64` rounded to an
    /// integer; LPCM rates are integral).
    pub sample_rate: u32,
    /// `numAudioChannels` — the real channel count.
    pub channel_count: u32,
    /// `constBitsPerChannel` — bits per sample.
    pub bits_per_channel: u32,
    /// `formatSpecificFlags` — CoreAudio LPCM flags (bit 0 = float, bit 1 =
    /// big-endian, bit 2 = signed integer, …).
    pub format_flags: u32,
}

impl Audio {
    /// Decode the `AudioSampleEntry` base plus any QuickTime v1/v2 sound sample
    /// description extension. The v1 extension carries no field a consumer needs
    /// (it is skipped); the v2 extension's real rate / channels / format is
    /// returned as [`SoundV2`]. Plain [`Decode`] discards the extension.
    ///
    /// Round-trip caveat: the version-2 `constBytesPerAudioPacket` and
    /// `constLPCMFramesPerAudioPacket` fields are NOT retained — [`SoundV2`]
    /// keeps only rate / channels / bits / flags — and [`Self::encode_with_v2`]
    /// reconstructs them as `channels * bits/8` and `1`. A `decode`→`encode`
    /// round-trip therefore reproduces them exactly only for uncompressed PCM
    /// with an integer number of bytes per sample and one frame per packet
    /// (which is what a version-2 `lpcm` entry is).
    pub fn decode_with_v2<B: Buf>(buf: &mut B) -> Result<(Self, Option<SoundV2>)> {
        u32::decode(buf)?; // reserved
        u16::decode(buf)?; // reserved
        let data_reference_index = u16::decode(buf)?;
        let version = u16::decode(buf)?;
        u16::decode(buf)?; // reserved
        u32::decode(buf)?; // reserved
        let channel_count = u16::decode(buf)?;
        let sample_size = u16::decode(buf)?;
        u32::decode(buf)?; // pre-defined, reserved
        let sample_rate = FixedPoint::decode(buf)?;

        let v2 = match version {
            0 => None,
            1 => {
                // QuickTime sound sample description version 1.
                u64::decode(buf)?;
                u64::decode(buf)?;
                None
            }
            2 => {
                // QuickTime sound sample description version 2.
                u32::decode(buf)?; // sizeOfStructOnly
                let sample_rate = f64::from_bits(u64::decode(buf)?).round() as u32;
                let channel_count = u32::decode(buf)?;
                u32::decode(buf)?; // always7F000000
                let bits_per_channel = u32::decode(buf)?;
                let format_flags = u32::decode(buf)?;
                u32::decode(buf)?; // constBytesPerAudioPacket
                u32::decode(buf)?; // constLPCMFramesPerAudioPacket
                Some(SoundV2 {
                    sample_rate,
                    channel_count,
                    bits_per_channel,
                    format_flags,
                })
            }
            n => return Err(Error::UnknownQuicktimeVersion(n)),
        };

        Ok((
            Self {
                data_reference_index,
                channel_count,
                sample_size,
                sample_rate,
            },
            v2,
        ))
    }

    /// Encode the `AudioSampleEntry` base, emitting the version-2 sound sample
    /// description extension when `v2` is `Some` (the base fields then carry the
    /// QTFF placeholders and `version` becomes 2). `None` writes the version-0
    /// form — the shared [`Encode`] path.
    ///
    /// The v2 `constBytesPerAudioPacket` / `constLPCMFramesPerAudioPacket` fields
    /// are reconstructed as `channel_count * bits_per_channel/8` and `1` (they
    /// are not carried on [`SoundV2`]) — exact for uncompressed one-frame-per-
    /// packet PCM; see [`Self::decode_with_v2`].
    pub fn encode_with_v2<B: BufMut>(&self, v2: Option<&SoundV2>, buf: &mut B) -> Result<()> {
        0u32.encode(buf)?; // reserved
        0u16.encode(buf)?; // reserved
        self.data_reference_index.encode(buf)?;
        (if v2.is_some() { 2u16 } else { 0u16 }).encode(buf)?; // version
        0u16.encode(buf)?; // reserved
        0u32.encode(buf)?; // reserved
        self.channel_count.encode(buf)?;
        self.sample_size.encode(buf)?;
        0u32.encode(buf)?; // reserved
        self.sample_rate.encode(buf)?;

        if let Some(v2) = v2 {
            72u32.encode(buf)?; // sizeOfStructOnly (constant)
            (v2.sample_rate as f64).to_bits().encode(buf)?; // audioSampleRate
            v2.channel_count.encode(buf)?;
            0x7F00_0000u32.encode(buf)?; // always7F000000
            v2.bits_per_channel.encode(buf)?;
            v2.format_flags.encode(buf)?;
            (v2.channel_count * (v2.bits_per_channel / 8)).encode(buf)?; // constBytesPerAudioPacket
            1u32.encode(buf)?; // constLPCMFramesPerAudioPacket
        }

        Ok(())
    }
}

impl Encode for Audio {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.encode_with_v2(None, buf)
    }
}
impl Decode for Audio {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Self::decode_with_v2(buf)?.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A QuickTime **version-1** sound sample description: `decode_with_v2` must
    // consume the 16-byte v1 extension (samples/bytes-per-packet/frame/sample)
    // and discard it — returning the base fields with NO `SoundV2` (that is the
    // version-2 path). Distinct from `pcm::tests::test_pcm_v1_chnl`, which
    // exercises the PCM `pcmC`/`chnl` child boxes, not the sound-description
    // version.
    #[test]
    fn test_audio_v1_extension_is_skipped() {
        let bytes: &[u8] = &[
            0, 0, 0, 0, // reserved
            0, 0, // reserved
            0, 1, // data_reference_index
            0, 1, // version = 1
            0, 0, // reserved
            0, 0, 0, 0, // reserved
            0, 2, // channel_count
            0, 16, // sample_size
            0, 0, 0, 0, // pre-defined + reserved
            0xbb, 0x80, 0, 0, // sample_rate = 48000 << 16
            // version-1 extension (16 bytes) — consumed and discarded.
            0, 0, 4, 0, 0, 0, 0, 1, 0, 0, 0, 4, 0, 0, 0, 2,
        ];

        let (audio, v2) = Audio::decode_with_v2(&mut &bytes[..]).expect("version-1 entry decodes");
        assert!(v2.is_none(), "a version-1 entry yields no SoundV2");
        assert_eq!(audio.data_reference_index, 1);
        assert_eq!(audio.channel_count, 2);
        assert_eq!(audio.sample_size, 16);
        assert_eq!(audio.sample_rate.integer(), 48000);
    }
}
