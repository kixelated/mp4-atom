use crate::*;

/// TextConfigBox ('txtC'), configuration for a TextMetaDataSampleEntry ('mett').///
///
/// See ISO/IEC 14496-12:2026 Section 12.3.3.2.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TxtC {
    pub text_config: String,
}

impl AtomExt for TxtC {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"txtC");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        Ok(Self {
            text_config: String::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.text_config.as_str().encode(buf)
    }
}

/// TextMetaDataSampleEntry ('mett'), used for text-based timed metadata tracks.
///
/// See ISO/IEC 14496-12:2026 Section 12.3.3.2.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mett {
    pub metadata: MetaData,
    pub content_encoding: String,
    pub mime_format: String,
    pub btrt: Option<Btrt>,
    pub txtc: Option<TxtC>,
}

impl Atom for Mett {
    const KIND: FourCC = FourCC::new(b"mett");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let metadata = MetaData::decode(buf)?;

        let content_encoding = String::decode(buf)?;
        let mime_format = String::decode(buf)?;

        let mut btrt = None;
        let mut txtc = None;
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Btrt(atom) => btrt = atom.into(),
                Any::TxtC(atom) => txtc = atom.into(),
                unknown => Self::decode_unknown(&unknown)?,
            }
        }
        skip_trailing_padding(buf);

        Ok(Self {
            metadata,
            content_encoding,
            mime_format,
            btrt,
            txtc,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.metadata.encode(buf)?;

        self.content_encoding.as_str().encode(buf)?;
        self.mime_format.as_str().encode(buf)?;
        self.btrt.encode(buf)?;
        self.txtc.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mett_roundtrip() {
        let mett = Mett {
            metadata: MetaData {
                data_reference_index: 1,
            },
            content_encoding: "".into(),
            mime_format: "text/plain".into(),
            btrt: Some(Btrt {
                buffer_size_db: 1,
                max_bitrate: 2000,
                avg_bitrate: 400,
            }),
            txtc: Some(TxtC {
                text_config: "some config".into(),
            }),
        };

        let mut buf = Vec::new();
        mett.encode(&mut buf).unwrap();

        let decoded = Mett::decode(&mut buf.as_slice()).expect("failed to decode mett");
        assert_eq!(decoded, mett);
    }

    #[test]
    fn test_mett_roundtrip_no_config() {
        let mett = Mett {
            metadata: MetaData {
                data_reference_index: 1,
            },
            content_encoding: "application/zip".into(),
            mime_format: "text/html".into(),
            btrt: None,
            txtc: None,
        };

        let mut buf = Vec::new();
        mett.encode(&mut buf).unwrap();

        let decoded = Mett::decode(&mut buf.as_slice()).expect("failed to decode mett");
        assert_eq!(decoded, mett);
    }
}
