use crate::*;

/// MPEG-4 Part 2 Visual (ISO/IEC 14496-2) — the `mp4v` sample entry.
///
/// A `VisualSampleEntry` whose codec configuration rides in an MPEG-4 `esds`
/// (the same elementary-stream descriptor container as `mp4a`, here with a
/// visual object-type indication such as `0x20`). Common in older MP4/3GP files
/// (DivX/Xvid, MPEG-4 SP/ASP). Optional `pasp`/`colr`/`btrt`/`fiel` children are
/// carried like the other visual entries.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mp4v {
    pub visual: Visual,
    pub esds: Esds,
    pub btrt: Option<Btrt>,
    pub colr: Option<Colr>,
    pub pasp: Option<Pasp>,
    pub fiel: Option<Fiel>,
}

impl Atom for Mp4v {
    const KIND: FourCC = FourCC::new(b"mp4v");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let visual = Visual::decode(buf)?;

        let mut esds = None;
        let mut btrt = None;
        let mut colr = None;
        let mut pasp = None;
        let mut fiel = None;
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Esds(atom) => esds = Some(atom),
                Any::Btrt(atom) => btrt = Some(atom),
                Any::Colr(atom) => colr = Some(atom),
                Any::Pasp(atom) => pasp = Some(atom),
                Any::Fiel(atom) => fiel = Some(atom),
                unknown => Self::decode_unknown(&unknown)?,
            }
        }
        Ok(Mp4v {
            visual,
            esds: esds.ok_or(Error::MissingBox(Esds::KIND))?,
            btrt,
            colr,
            pasp,
            fiel,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.visual.encode(buf)?;
        self.esds.encode(buf)?;
        self.btrt.encode(buf)?;
        self.colr.encode(buf)?;
        self.pasp.encode(buf)?;
        self.fiel.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_esds() -> Esds {
        Esds {
            es_desc: esds::EsDescriptor {
                es_id: 1,
                // object_type_indication 0x20 = MPEG-4 Visual; stream_type 0x04
                // = VisualStream.
                dec_config: esds::DecoderConfig {
                    object_type_indication: 0x20,
                    stream_type: 0x04,
                    up_stream: 0,
                    buffer_size_db: Default::default(),
                    max_bitrate: 0,
                    avg_bitrate: 0,
                    // A representative MPEG-4 Visual DecoderSpecificInfo — the
                    // VisualObjectSequence / VideoObject start codes and VOL
                    // header. These bytes are opaque to the AAC field parse
                    // (`profile`/`freq_index`/`chan_conf` resolve to 0), but are
                    // preserved verbatim in `raw` so the config round-trips.
                    dec_specific: Some(esds::DecoderSpecific {
                        profile: 0,
                        freq_index: 0,
                        chan_conf: 0,
                        raw: vec![0x00, 0x00, 0x01, 0xb0, 0x01, 0x00, 0x00, 0x01, 0xb5],
                    }),
                },
                sl_config: esds::SLConfig::default(),
            },
        }
    }

    #[test]
    fn test_mp4v_roundtrip() {
        let expected = Mp4v {
            visual: Visual {
                data_reference_index: 1,
                width: 640,
                height: 480,
                horizresolution: 0x48.into(),
                vertresolution: 0x48.into(),
                frame_count: 1,
                compressor: "".into(),
                depth: 24,
            },
            esds: sample_esds(),
            btrt: None,
            colr: None,
            pasp: None,
            fiel: None,
        };

        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Mp4v::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
        assert_eq!(decoded.visual.width, 640);
        assert_eq!(decoded.visual.height, 480);
        assert_eq!(decoded.esds.es_desc.dec_config.object_type_indication, 0x20);
    }
}
