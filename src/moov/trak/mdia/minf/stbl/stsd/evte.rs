use crate::*;

/// `EventMessageSampleEntry` ('evte'), used by CMAF/DASH "Event Message tracks"
/// to carry timed [`Emsg`] boxes as samples instead of inline in fragments.
///
/// See ISO/IEC 23001-18:2022, "Carriage of event messages in the ISO base media
/// file format", Section 7.2. Each sample is itself contains one or more
/// [`Emib`] boxes or a single [`Emeb`] box.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Evte {
    pub metadata: MetaData,
    pub btrt: Option<Btrt>,
    pub silb: Option<Silb>,
}

impl Atom for Evte {
    const KIND: FourCC = FourCC::new(b"evte");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let metadata = MetaData::decode(buf)?;

        let mut btrt = None;
        let mut silb = None;
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Btrt(atom) => btrt = atom.into(),
                Any::Silb(atom) => silb = atom.into(),
                unknown => Self::decode_unknown(&unknown)?,
            }
        }
        skip_trailing_padding(buf);

        Ok(Self {
            metadata,
            btrt,
            silb,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.metadata.encode(buf)?;
        self.btrt.encode(buf)?;
        self.silb.encode(buf)?;
        Ok(())
    }
}

/// A single scheme entry in a [`Silb`] box.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SchemeIdEntry {
    pub scheme_id_uri: String,
    pub value: String,
    /// Whether the track contains at least one event instance of this scheme.
    pub atleast_once: bool,
}

/// `SchemeIdListBox` ('silb'), an optional child of [`Evte`] declaring the
/// event schemes that may appear in the track.
///
/// See ISO/IEC 23001-18:2022 Section 7.3.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Silb {
    pub schemes: Vec<SchemeIdEntry>,
    /// Whether the track may contain schemes other than the ones listed here.
    pub other_schemes: bool,
}

impl AtomExt for Silb {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"silb");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let number_of_schemes = u32::decode(buf)?;
        // Each entry is at least 3 bytes (two null-terminated strings and a
        // flags byte); reject counts that cannot possibly fit before
        // allocating, as with the FLAC Vorbis comment field count (#154).
        if number_of_schemes as u64 > buf.remaining() as u64 / 3 {
            return Err(Error::OutOfBounds);
        }

        let mut schemes = Vec::with_capacity((number_of_schemes as usize).min(16));
        for _ in 0..number_of_schemes {
            let scheme_id_uri = String::decode(buf)?;
            let value = String::decode(buf)?;
            let flags = u8::decode(buf)?;
            schemes.push(SchemeIdEntry {
                scheme_id_uri,
                value,
                atleast_once: flags & 0x1 != 0,
            });
        }

        let other_schemes_byte = u8::decode(buf)?;

        Ok(Self {
            schemes,
            other_schemes: other_schemes_byte & 0x1 != 0,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        (self.schemes.len() as u32).encode(buf)?;
        for scheme in &self.schemes {
            scheme.scheme_id_uri.as_str().encode(buf)?;
            scheme.value.as_str().encode(buf)?;
            let flags: u8 = if scheme.atleast_once { 0x1 } else { 0 };
            flags.encode(buf)?;
        }

        let other_schemes_byte: u8 = if self.other_schemes { 0x1 } else { 0 };
        other_schemes_byte.encode(buf)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A bare `evte` sample entry: reserved(6) + data_reference_index(2), no
    // optional child boxes (ISO/IEC 23001-18:2022).
    const ENCODED_EVTE: &[u8] = &[
        0x00, 0x00, 0x00, 0x10, 0x65, 0x76, 0x74, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01,
    ];

    fn decoded_evte() -> Evte {
        Evte {
            metadata: MetaData {
                data_reference_index: 1,
            },
            btrt: None,
            silb: None,
        }
    }

    #[test]
    fn test_evte_decode() {
        let buf = &mut std::io::Cursor::new(ENCODED_EVTE);
        let evte = Evte::decode(buf).expect("failed to decode evte");
        assert_eq!(evte, decoded_evte());
    }

    #[test]
    fn test_evte_encode() {
        let mut buf = Vec::new();
        decoded_evte().encode(&mut buf).unwrap();
        assert_eq!(buf.as_slice(), ENCODED_EVTE);
    }

    #[test]
    fn test_evte_via_codec() {
        let evte = decoded_evte();

        let mut buf = Vec::new();
        Codec::from(evte.clone()).encode(&mut buf).unwrap();
        assert_eq!(buf.as_slice(), ENCODED_EVTE);

        let decoded = Codec::decode(&mut buf.as_slice()).expect("failed to decode evte codec");
        assert_eq!(decoded, Codec::Evte(evte));
    }

    #[test]
    fn test_silb_roundtrip() {
        let silb = Silb {
            schemes: vec![
                SchemeIdEntry {
                    scheme_id_uri: "urn:scte:scte35:2014:xml+bin".into(),
                    value: "1".into(),
                    atleast_once: true,
                },
                SchemeIdEntry {
                    scheme_id_uri: "urn:mpeg:dash:event:2012".into(),
                    value: "".into(),
                    atleast_once: false,
                },
            ],
            other_schemes: true,
        };

        let mut buf = Vec::new();
        silb.encode(&mut buf).unwrap();

        let decoded = Silb::decode(&mut buf.as_slice()).expect("failed to decode silb");
        assert_eq!(decoded, silb);
    }

    #[test]
    fn test_evte_with_btrt_and_silb_roundtrip() {
        let evte = Evte {
            metadata: MetaData {
                data_reference_index: 1,
            },
            btrt: Some(Btrt {
                buffer_size_db: 0,
                max_bitrate: 1000,
                avg_bitrate: 500,
            }),
            silb: Some(Silb {
                schemes: vec![SchemeIdEntry {
                    scheme_id_uri: "urn:mpeg:dash:event:2012".into(),
                    value: "".into(),
                    atleast_once: true,
                }],
                other_schemes: false,
            }),
        };

        let mut buf = Vec::new();
        evte.encode(&mut buf).unwrap();

        let decoded = Evte::decode(&mut buf.as_slice()).expect("failed to decode evte");
        assert_eq!(decoded, evte);
    }

    // Regression-style guard, mirroring the FLAC Vorbis comment fix (#154):
    // a huge number_of_schemes must fail cleanly rather than trying to
    // allocate an enormous Vec upfront.
    #[test]
    fn test_silb_huge_count() {
        let mut buf = Vec::new();
        u32::MAX.encode(&mut buf).unwrap(); // number_of_schemes

        assert!(matches!(
            Silb::decode_body_ext(&mut buf.as_slice(), ()),
            Err(Error::OutOfBounds)
        ));
    }
}
