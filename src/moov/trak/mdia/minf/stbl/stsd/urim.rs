use crate::*;

/// URIBox ('uri '), the URI identifying the metadata scheme for a URIMetaSampleEntry ('urim').
///
/// See ISO/IEC 14496-12:2026 Section 12.3.3.2.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Uri {
    pub the_uri: String,
}

impl AtomExt for Uri {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"uri ");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        Ok(Self {
            the_uri: String::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.the_uri.as_str().encode(buf)
    }
}

/// URIInitBox ('uriI'), optional initialization data for a URIMetaSampleEntry ('urim').
///
/// See ISO/IEC 14496-12:2026 Section 12.3.3.2.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UriI {
    pub uri_initialization_data: Vec<u8>,
}

impl AtomExt for UriI {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"uriI");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        Ok(Self {
            uri_initialization_data: Vec::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.uri_initialization_data.encode(buf)
    }
}

/// URIMetaSampleEntry ('urim'), used for URI-based timed metadata tracks.
///
/// See ISO/IEC 14496-12:2026 Section 12.3.3.2.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Urim {
    pub metadata: MetaData,
    pub the_label: Uri,
    pub init: Option<UriI>,
    pub btrt: Option<Btrt>,
}

impl Atom for Urim {
    const KIND: FourCC = FourCC::new(b"urim");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let metadata = MetaData::decode(buf)?;

        let the_label = Uri::decode(buf)?;

        let mut init = None;
        let mut btrt = None;
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::UriI(atom) => init = atom.into(),
                Any::Btrt(atom) => btrt = atom.into(),
                unknown => Self::decode_unknown(&unknown)?,
            }
        }
        skip_trailing_padding(buf);

        Ok(Self {
            metadata,
            the_label,
            init,
            btrt,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.metadata.encode(buf)?;

        self.the_label.encode(buf)?;
        self.init.encode(buf)?;
        self.btrt.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urim_roundtrip() {
        let urim = Urim {
            metadata: MetaData {
                data_reference_index: 1,
            },
            the_label: Uri {
                the_uri: "urn:example:uri-metadata".into(),
            },
            init: Some(UriI {
                uri_initialization_data: vec![1, 2, 3, 4],
            }),
            btrt: Some(Btrt {
                buffer_size_db: 1,
                max_bitrate: 2000,
                avg_bitrate: 400,
            }),
        };

        let mut buf = Vec::new();
        urim.encode(&mut buf).unwrap();

        let decoded = Urim::decode(&mut buf.as_slice()).expect("failed to decode urim");
        assert_eq!(decoded, urim);
    }

    #[test]
    fn test_urim_roundtrip_no_init() {
        let urim = Urim {
            metadata: MetaData {
                data_reference_index: 1,
            },
            the_label: Uri {
                the_uri: "https://example.com/schema.xsd".into(),
            },
            init: None,
            btrt: None,
        };

        let mut buf = Vec::new();
        urim.encode(&mut buf).unwrap();

        let decoded = Urim::decode(&mut buf.as_slice()).expect("failed to decode urim");
        assert_eq!(decoded, urim);
    }

    #[test]
    fn test_urim_missing_uri() {
        let mut buf = Vec::new();
        0u32.encode(&mut buf).unwrap(); // reserved
        0u16.encode(&mut buf).unwrap(); // reserved
        1u16.encode(&mut buf).unwrap(); // data_reference_index

        let err = Urim::decode_body(&mut buf.as_slice()).unwrap_err();
        assert!(matches!(err, Error::OutOfBounds));
    }
}
