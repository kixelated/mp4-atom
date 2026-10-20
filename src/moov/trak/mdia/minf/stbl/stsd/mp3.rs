use crate::*;

// See the QuickTime File Format Specification, "Sound Sample Description
// Formats" (the `.mp3` / `kAudioFormatMPEGLayer3` format identifier).

/// MPEG-1/2 Audio Layer III (MP3) sample entry, as used by QuickTime and
/// other tools that store MP3 directly rather than wrapping it in `mp4a`/`esds`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mp3 {
    pub audio: Audio,
}

impl Atom for Mp3 {
    const KIND: FourCC = FourCC::new(b".mp3");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let audio = Audio::decode(buf)?;

        while let Some(atom) = Any::decode_maybe(buf)? {
            Self::decode_unknown(&atom)?;
        }
        skip_trailing_padding(buf);

        Ok(Self { audio })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.audio.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENCODED_MP3: &[u8] = &[
        0x00, 0x00, 0x00, 0x24, 0x2e, 0x6d, 0x70, 0x33, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x10, 0x00, 0x00,
        0x00, 0x00, 0xac, 0x44, 0x00, 0x00,
    ];

    #[test]
    fn test_mp3_decode() {
        let buf: &mut std::io::Cursor<&[u8]> = &mut std::io::Cursor::new(ENCODED_MP3);

        let mp3 = Mp3::decode(buf).expect("failed to decode .mp3");

        assert_eq!(
            mp3,
            Mp3 {
                audio: Audio {
                    data_reference_index: 1,
                    channel_count: 2,
                    sample_size: 16,
                    sample_rate: 44100.into(),
                }
            }
        );
    }

    #[test]
    fn test_mp3_encode() {
        let mp3 = Mp3 {
            audio: Audio {
                data_reference_index: 1,
                channel_count: 2,
                sample_size: 16,
                sample_rate: 44100.into(),
            },
        };

        let mut buf = Vec::new();
        mp3.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_MP3);
    }
}
