use crate::*;

/// XMLMetaDataSampleEntry ('metx'), used for XML-based timed metadata tracks.
///
/// See ISO/IEC 14496-12 Section 12.3.3.2.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Metx {
    pub metadata: MetaData,
    pub content_encoding: String,
    pub namespace: String,
    pub schema_location: String,
    pub btrt: Option<Btrt>,
}

impl Atom for Metx {
    const KIND: FourCC = FourCC::new(b"metx");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let metadata = MetaData::decode(buf)?;

        let content_encoding = String::decode(buf)?;
        let namespace = String::decode(buf)?;
        let schema_location = String::decode(buf)?;

        let mut btrt = None;
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Btrt(atom) => btrt = atom.into(),
                unknown => Self::decode_unknown(&unknown)?,
            }
        }
        skip_trailing_padding(buf);

        Ok(Self {
            metadata,
            content_encoding,
            namespace,
            schema_location,
            btrt,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.metadata.encode(buf)?;

        self.content_encoding.as_str().encode(buf)?;
        self.namespace.as_str().encode(buf)?;
        self.schema_location.as_str().encode(buf)?;
        self.btrt.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metx_roundtrip() {
        let metx = Metx {
            metadata: MetaData {
                data_reference_index: 1,
            },
            content_encoding: "".into(),
            namespace: "urn:example:namespace".into(),
            schema_location: "https://example.com/schema.xsd".into(),
            btrt: Some(Btrt {
                buffer_size_db: 1,
                max_bitrate: 2000,
                avg_bitrate: 400,
            }),
        };

        let mut buf = Vec::new();
        metx.encode(&mut buf).unwrap();

        let decoded = Metx::decode(&mut buf.as_slice()).expect("failed to decode metx");
        assert_eq!(decoded, metx);
    }

    #[test]
    fn test_metx_roundtrip_no_schema_location() {
        let metx = Metx {
            metadata: MetaData {
                data_reference_index: 1,
            },
            content_encoding: "".into(),
            namespace: "https://example.com/ns".into(),
            schema_location: "".into(),
            btrt: None,
        };

        let mut buf = Vec::new();
        metx.encode(&mut buf).unwrap();

        let decoded = Metx::decode(&mut buf.as_slice()).expect("failed to decode metx");
        assert_eq!(decoded, metx);
    }
}
