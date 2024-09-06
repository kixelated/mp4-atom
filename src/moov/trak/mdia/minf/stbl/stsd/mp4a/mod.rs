use crate::*;

// We're trying not to pollute the global namespace
pub mod esds;
pub use esds::Esds;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mp4a {
    pub data_reference_index: u16,
    pub channelcount: u16,
    pub samplesize: u16,
    pub samplerate: FixedPoint<u16>,
    pub esds: Option<Esds>,
}

impl Default for Mp4a {
    fn default() -> Self {
        Self {
            data_reference_index: 0,
            channelcount: 2,
            samplesize: 16,
            samplerate: 48000.into(),
            esds: Some(Esds::default()),
        }
    }
}

impl Atom for Mp4a {
    const KIND: FourCC = FourCC::new(b"mp4a");

    fn decode_body(buf: &mut Bytes) -> Result<Self> {
        u32::decode(buf)?; // reserved
        u16::decode(buf)?; // reserved
        let data_reference_index = buf.decode()?;
        let version = u16::decode(buf)?;
        u16::decode(buf)?; // reserved
        u32::decode(buf)?; // reserved
        let channelcount = buf.decode()?;
        let samplesize = buf.decode()?;
        u32::decode(buf)?; // pre-defined, reserved
        let samplerate = buf.decode()?;

        if version == 1 {
            // Skip QTFF
            u64::decode(buf)?;
            u64::decode(buf)?;
        }

        let mut esds = None;

        // Find esds in mp4a or wave
        while let Some(atom) = buf.decode()? {
            match atom {
                Any::Esds(atom) => esds = atom.into(),
                _ => tracing::warn!("unknown atom: {:?}", atom),
            }
        }

        Ok(Mp4a {
            data_reference_index,
            channelcount,
            samplesize,
            samplerate,
            esds,
        })
    }

    fn encode_body(&self, buf: &mut BytesMut) -> Result<()> {
        0u32.encode(buf)?; // reserved
        0u16.encode(buf)?; // reserved
        self.data_reference_index.encode(buf)?;
        0u16.encode(buf)?; // version
        0u16.encode(buf)?; // reserved
        0u32.encode(buf)?; // reserved
        self.channelcount.encode(buf)?;
        self.samplesize.encode(buf)?;
        0u32.encode(buf)?; // reserved
        self.samplerate.encode(buf)?;

        self.esds.encode(buf)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mp4a() {
        let expected = Mp4a {
            data_reference_index: 1,
            channelcount: 2,
            samplesize: 16,
            samplerate: 48000.into(),
            esds: Some(Esds {
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
            }),
        };
        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let decoded = Mp4a::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_mp4a_no_esds() {
        let expected = Mp4a {
            data_reference_index: 1,
            channelcount: 2,
            samplesize: 16,
            samplerate: 48000.into(),
            esds: None,
        };
        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let decoded = Mp4a::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
