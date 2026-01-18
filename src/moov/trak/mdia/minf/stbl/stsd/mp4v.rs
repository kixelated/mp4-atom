use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mp4v {
    pub visual: Visual,
    pub esds: Esds,
    pub btrt: Option<Btrt>,
    pub taic: Option<Taic>,
}

impl Atom for Mp4v {
    const KIND: FourCC = FourCC::new(b"mp4v");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let visual = Visual::decode(buf)?;

        let mut btrt = None;
        let mut esds = None;
        let mut taic = None;

        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Btrt(atom) => btrt = atom.into(),
                Any::Esds(atom) => esds = atom.into(),
                Any::Taic(atom) => taic = atom.into(),
                unknown => Self::decode_unknown(&unknown)?,
            }
        }

        Ok(Mp4v {
            visual,
            esds: esds.ok_or(Error::MissingBox(Esds::KIND))?,
            btrt,
            taic,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.visual.encode(buf)?;
        self.esds.encode(buf)?;
        self.btrt.encode(buf)?;
        self.taic.encode(buf)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mp4v() {
        let expected = Mp4v {
            visual: Visual {
                data_reference_index: 0,
                width: 100,
                height: 80,
                horizresolution: 72.into(),
                vertresolution: 48.into(),
                frame_count: 3,
                compressor: "test".into(),
                depth: 10,
            },
            esds: Esds {
                es_desc: esds::EsDescriptor {
                    es_id: 2,
                    dec_config: esds::DecoderConfig {
                        object_type_indication: 0x20,
                        stream_type: 0x04,
                        up_stream: 0,
                        buffer_size_db: Default::default(),
                        max_bitrate: 4000000,
                        avg_bitrate: 2000000,
                        // TODO: rework this to be a video version
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
        let decoded = Mp4v::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
