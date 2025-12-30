use crate::*;

// We're trying not to pollute the global namespace
pub mod esds;
pub use esds::Esds;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mp4a {
    pub audio: Audio,
    pub esds: Esds,
    pub btrt: Option<Btrt>,
    pub taic: Option<Taic>,
}

impl Atom for Mp4a {
    const KIND: FourCC = FourCC::new(b"mp4a");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let audio = Audio::decode(buf)?;

        let mut btrt = None;
        let mut esds = None;
        let mut taic = None;

        // Find esds in mp4a or wave
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Btrt(atom) => btrt = atom.into(),
                Any::Esds(atom) => esds = atom.into(),
                Any::Taic(atom) => taic = atom.into(),
                unknown => Self::decode_unknown(&unknown)?,
            }
        }

        Ok(Mp4a {
            audio,
            esds: esds.ok_or(Error::MissingBox(Esds::KIND))?,
            btrt,
            taic,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.audio.encode(buf)?;
        self.esds.encode(buf)?;
        if self.btrt.is_some() {
            self.btrt.encode(buf)?;
        }
        if self.taic.is_some() {
            self.taic.encode(buf)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mp4a() {
        let expected = Mp4a {
            audio: Audio {
                data_reference_index: 1,
                channel_count: 2,
                sample_size: 16,
                sample_rate: 48000.into(),
            },
            esds: Esds {
                es_desc: esds::EsDescriptor {
                    es_id: 2,
                    dec_config: esds::DecoderConfig {
                        object_type_indication: 0x40,
                        stream_type: 0x05,
                        up_stream: 0,
                        buffer_size_db: Default::default(),
                        max_bitrate: 67695,
                        avg_bitrate: 67695,
                        dec_specific: esds::DecoderSpecific {
                            profile: 2,
                            freq_index: 4,
                            chan_conf: 2,
                        },
                    },
                    sl_config: esds::SLConfig::default(),
                },
            },
            btrt: Some(Btrt {
                buffer_size_db: 1,
                max_bitrate: 2,
                avg_bitrate: 3,
            }),
            taic: None,
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Mp4a::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_mp4a_with_taic() {
        let expected = Mp4a {
            audio: Audio {
                data_reference_index: 1,
                channel_count: 2,
                sample_size: 16,
                sample_rate: 48000.into(),
            },
            esds: Esds {
                es_desc: esds::EsDescriptor {
                    es_id: 2,
                    dec_config: esds::DecoderConfig {
                        object_type_indication: 0x40,
                        stream_type: 0x05,
                        up_stream: 0,
                        buffer_size_db: Default::default(),
                        max_bitrate: 67695,
                        avg_bitrate: 67695,
                        dec_specific: esds::DecoderSpecific {
                            profile: 2,
                            freq_index: 4,
                            chan_conf: 2,
                        },
                    },
                    sl_config: esds::SLConfig::default(),
                },
            },
            btrt: None,
            taic: Some(Taic {
                time_uncertainty: 1,
                clock_resolution: 2,
                clock_drift_rate: 4,
                clock_type: ClockType::CanSync,
            }),
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Mp4a::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
