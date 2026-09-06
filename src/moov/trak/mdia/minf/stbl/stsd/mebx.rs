use crate::*;

/// Metadata Key Declaration Box ('keyd').
///
/// Holds the key namespace and key value of that namespace for the given
/// values.
///
/// See ISO/IEC 14496-12:2026 Section 12.8.4.4.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Keyd {
    pub key_namespace: FourCC,
    pub key_value: Vec<u8>,
}

impl Atom for Keyd {
    const KIND: FourCC = FourCC::new(b"keyd");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let key_namespace = FourCC::decode(buf)?;
        let key_value = Vec::decode(buf)?;
        Ok(Self {
            key_namespace,
            key_value,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.key_namespace.encode(buf)?;
        self.key_value.encode(buf)
    }
}

/// The value type declared by a [`Dtyp`] atom.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DataType {
    /// `datatype_namespace` `0`: a well-known type code shared with the
    /// iTunes `data` atom (e.g. `1` = UTF-8, `21` = BE signed integer, `23` =
    /// BE float32) -- see the [well-known types](https://developer.apple.com/documentation/quicktime-file-format/well-known_types) table.
    WellKnown(u32),
    /// `datatype_namespace` `1`: a namespace-specific type name (a
    /// case-sensitive reverse-DNS string, e.g. for a custom/structured value
    /// type not covered by a well-known type).
    Custom(String),
    /// A `datatype_namespace` this crate doesn't interpret (reserved for a
    /// future revision of the spec, or a foreign metadata standard's own
    /// numbering/naming scheme); the raw bytes are preserved as-is.
    Unknown(u32, Vec<u8>),
}

/// Metadata Datatype Definition Atom ('dtyp').
///
/// Declares the value type of a metadata key. Optional -- a key with no
/// `dtyp` has an implicit/unspecified type.
///
/// Apple/QuickTime-specific: not part of ISO/IEC 14496-12's own
/// `MetadataKeyBox` (which instead only defines `keyd`/`loca`/`setu`), but a
/// commonly-written extension child under the same box.
///
/// See Apple's [Metadata datatype definition atom](https://developer.apple.com/documentation/quicktime-file-format/metadata_datatype_definition_atom).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dtyp {
    pub data_type: DataType,
}

impl Atom for Dtyp {
    const KIND: FourCC = FourCC::new(b"dtyp");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let namespace = u32::decode(buf)?;
        let data_type = match namespace {
            0 => DataType::WellKnown(u32::decode(buf)?),
            // A case-sensitive UTF-8 string without a null terminator,
            // filling the rest of the atom.
            1 => {
                let remaining = buf.remaining();
                let bytes = buf.slice(remaining).to_vec();
                buf.advance(remaining);
                DataType::Custom(
                    String::from_utf8(bytes)
                        .map_err(|err| Error::InvalidString(err.to_string()))?,
                )
            }
            _ => {
                let remaining = buf.remaining();
                let raw = buf.slice(remaining).to_vec();
                buf.advance(remaining);
                DataType::Unknown(namespace, raw)
            }
        };
        Ok(Self { data_type })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        match &self.data_type {
            DataType::WellKnown(code) => {
                0u32.encode(buf)?;
                code.encode(buf)
            }
            DataType::Custom(name) => {
                1u32.encode(buf)?;
                buf.append_slice(name.as_bytes());
                Ok(())
            }
            DataType::Unknown(namespace, raw) => {
                namespace.encode(buf)?;
                buf.append_slice(raw);
                Ok(())
            }
        }
    }
}

/// Metadata Locale Box ('loca').
///
/// Tags a key declaration as applying only to a specific locale, so a track
/// can declare multiple locale-specific variants of the same key. Absent
/// means the key applies to all locales.
///
/// See ISO/IEC 14496-12:2026 Section 12.8.4.5.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Loca {
    /// A language tag complying with IETF BCP 47 (e.g. `en-US`).
    pub locale: String,
}

impl Atom for Loca {
    const KIND: FourCC = FourCC::new(b"loca");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Self {
            locale: String::decode(buf)?,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.locale.as_str().encode(buf)
    }
}

/// Metadata Setup Box ('setu').
///
/// Carries setup data needed to interpret values for some `key_namespace`s
/// (e.g. the boxes that would otherwise appear in the sample entry, for the
/// case of multiplexed timed metadata). The contents are namespace-defined,
/// so this crate preserves them as opaque bytes.
///
/// See ISO/IEC 14496-12:2026 Section 12.8.4.6.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Setu {
    pub data: Vec<u8>,
}

impl Atom for Setu {
    const KIND: FourCC = FourCC::new(b"setu");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Self {
            data: Vec::decode(buf)?,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.data.encode(buf)
    }
}

/// A Metadata Key Box: one local-key declaration inside a [`Mebx`] sample
/// entry's `keys` table (`MetadataKeyTableBox`).
///
/// Unlike every other atom in this crate, its own box type isn't a fixed
/// FourCC -- it's the arbitrary `local_key_id` the muxer chose for this key,
/// which per-sample metadata items reference by using that same FourCC as
/// their own box type.
/// `local_key_id` `0xFFFFFFFF` is reserved and must not appear here.
///
/// See ISO/IEC 14496-12:2026 Section 12.8.4.3.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MebxKey {
    pub local_key_id: FourCC,
    pub keyd: Keyd,
    pub dtyp: Option<Dtyp>,
    pub loca: Option<Loca>,
    pub setu: Option<Setu>,
}

impl MebxKey {
    fn decode_maybe<B: Buf>(buf: &mut B) -> Result<Option<Self>> {
        let Some(header) = Header::decode_maybe(buf)? else {
            return Ok(None);
        };
        if u32::from(header.kind) == 0xFFFF_FFFF {
            // `local_key_id` 0xFFFFFFFF is reserved and must never appear.
            return Err(Error::Reserved);
        }
        let size = header.size.unwrap_or(buf.remaining());
        if size > buf.remaining() {
            return Ok(None);
        }

        let mut body = buf.slice(size);

        let mut keyd = None;
        let mut dtyp = None;
        let mut loca = None;
        let mut setu = None;
        while let Some(atom) = Any::decode_maybe(&mut body)? {
            match atom {
                Any::Keyd(atom) => keyd = atom.into(),
                Any::Dtyp(atom) => dtyp = atom.into(),
                Any::Loca(atom) => loca = atom.into(),
                Any::Setu(atom) => setu = atom.into(),
                unknown => crate::decode_unknown(&unknown, header.kind)?,
            }
        }
        buf.advance(size);

        Ok(Some(Self {
            local_key_id: header.kind,
            keyd: keyd.ok_or(Error::MissingBox(Keyd::KIND))?,
            dtyp,
            loca,
            setu,
        }))
    }

    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let start = buf.len();
        0u32.encode(buf)?; // size placeholder
        self.local_key_id.encode(buf)?;
        self.keyd.encode(buf)?;
        self.dtyp.encode(buf)?;
        self.loca.encode(buf)?;
        self.setu.encode(buf)?;

        let size: u32 = (buf.len() - start)
            .try_into()
            .map_err(|_| Error::TooLarge(self.local_key_id))?;
        buf.set_slice(start, &size.to_be_bytes());
        Ok(())
    }
}

// Mebx's `keys` box shares the FourCC 'keys' with the unrelated ISO
// meta/ilst "Metadata item keys atom" (`crate::meta::Keys`, a FullBox with a
// fixed `entry_count`-prefixed layout), which IS registered in the global
// `Any` table. Deliberately use a local constant here rather than that
// type's `KIND` -- the two boxes are unrelated and must stay decoupled, even
// though they happen to share the same literal FourCC value.
const KEYS_KIND: FourCC = FourCC::new(b"keys");

fn decode_keys<B: Buf>(header: &Header, buf: &mut B) -> Result<Vec<MebxKey>> {
    let size = header.size.unwrap_or(buf.remaining());
    let mut body = buf.slice(size);

    let mut keys = Vec::new();
    while let Some(key) = MebxKey::decode_maybe(&mut body)? {
        keys.push(key);
    }
    buf.advance(size);
    Ok(keys)
}

fn encode_keys<B: BufMut>(keys: &[MebxKey], buf: &mut B) -> Result<()> {
    let start = buf.len();
    0u32.encode(buf)?; // size placeholder
    KEYS_KIND.encode(buf)?;
    for key in keys {
        key.encode(buf)?;
    }

    let size: u32 = (buf.len() - start)
        .try_into()
        .map_err(|_| Error::TooLarge(KEYS_KIND))?;
    buf.set_slice(start, &size.to_be_bytes());
    Ok(())
}

/// BoxedMetadataSampleEntry ('mebx'), used for multiplexed/"boxed" timed
/// metadata tracks (e.g. per-sample GPS or camera metadata with multiple
/// typed keys sharing a single track).
///
/// Unlike [`Mett`]/[`Metx`]/[`Urim`] (single-value metadata streams), each
/// sample is itself a concatenation of one or more value boxes, one per key
/// declared in `keys` -- the sample item's own box type is the key's
/// `local_key_id`.
///
/// See ISO/IEC 14496-12:2026 Section 12.8.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mebx {
    pub metadata: MetaData,
    pub keys: Vec<MebxKey>,
    pub btrt: Option<Btrt>,
}

impl Atom for Mebx {
    const KIND: FourCC = FourCC::new(b"mebx");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let metadata = MetaData::decode(buf)?;

        let mut keys = None;
        let mut btrt = None;
        while let Some(header) = Header::decode_maybe(buf)? {
            let size = header.size.unwrap_or(buf.remaining());
            if size > buf.remaining() {
                break;
            }
            if header.kind == KEYS_KIND {
                keys = Some(decode_keys(&header, buf)?);
                continue;
            }
            match Any::decode_atom(&header, buf)? {
                Any::Btrt(atom) => btrt = Some(atom),
                unknown => Self::decode_unknown(&unknown)?,
            }
        }
        skip_trailing_padding(buf);

        Ok(Self {
            metadata,
            keys: keys.ok_or(Error::MissingBox(KEYS_KIND))?,
            btrt,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.metadata.encode(buf)?;
        encode_keys(&self.keys, buf)?;
        self.btrt.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_keys() -> Vec<MebxKey> {
        vec![
            MebxKey {
                local_key_id: FourCC::new(b"\0\0\0\x01"),
                keyd: Keyd {
                    key_namespace: FourCC::new(b"mdta"),
                    key_value: b"com.apple.quicktime.location.ISO6709".to_vec(),
                },
                dtyp: Some(Dtyp {
                    data_type: DataType::WellKnown(1), // UTF-8
                }),
                loca: None,
                setu: None,
            },
            MebxKey {
                local_key_id: FourCC::new(b"\0\0\0\x02"),
                keyd: Keyd {
                    key_namespace: FourCC::new(b"mdta"),
                    key_value: b"com.example.custom-value".to_vec(),
                },
                dtyp: Some(Dtyp {
                    data_type: DataType::Custom("com.example.custom-type".into()),
                }),
                loca: Some(Loca {
                    locale: "en-US".into(),
                }),
                setu: Some(Setu {
                    data: vec![1, 2, 3, 4],
                }),
            },
        ]
    }

    #[test]
    fn test_keys_roundtrip() {
        let keys = sample_keys();

        let mut buf = Vec::new();
        encode_keys(&keys, &mut buf).unwrap();

        let mut slice = buf.as_slice();
        let header = Header::decode(&mut slice).unwrap();
        let decoded = decode_keys(&header, &mut slice).expect("failed to decode keys");
        assert_eq!(decoded, keys);
    }

    #[test]
    fn test_mebx_roundtrip() {
        let mebx = Mebx {
            metadata: MetaData {
                data_reference_index: 1,
            },
            keys: sample_keys(),
            btrt: Some(Btrt {
                buffer_size_db: 1,
                max_bitrate: 2000,
                avg_bitrate: 400,
            }),
        };

        let mut buf = Vec::new();
        mebx.encode(&mut buf).unwrap();

        let decoded = Mebx::decode(&mut buf.as_slice()).expect("failed to decode mebx");
        assert_eq!(decoded, mebx);
    }

    #[test]
    fn test_mebx_roundtrip_no_btrt() {
        let mebx = Mebx {
            metadata: MetaData {
                data_reference_index: 1,
            },
            keys: vec![],
            btrt: None,
        };

        let mut buf = Vec::new();
        mebx.encode(&mut buf).unwrap();

        let decoded = Mebx::decode(&mut buf.as_slice()).expect("failed to decode mebx");
        assert_eq!(decoded, mebx);
    }

    #[test]
    fn test_mebx_missing_keys() {
        let mut buf = Vec::new();
        0u32.encode(&mut buf).unwrap(); // reserved
        0u16.encode(&mut buf).unwrap(); // reserved
        1u16.encode(&mut buf).unwrap(); // data_reference_index

        let err = Mebx::decode_body(&mut buf.as_slice()).unwrap_err();
        assert!(matches!(err, Error::MissingBox(kind) if kind == KEYS_KIND));
    }

    #[test]
    fn test_mebx_key_rejects_reserved_local_key_id() {
        // `local_key_id` 0xFFFFFFFF is reserved and must
        // never appear as a key's own box type.
        let mut buf = Vec::new();
        8u32.encode(&mut buf).unwrap(); // size (header only, empty body)
        0xFFFF_FFFFu32.encode(&mut buf).unwrap(); // local_key_id (reserved)

        let err = MebxKey::decode_maybe(&mut buf.as_slice()).unwrap_err();
        assert!(matches!(err, Error::Reserved));
    }

    #[test]
    fn test_keyd_non_utf8_key_value_roundtrip() {
        // key_value is namespace-defined raw bytes, e.g. for the 'uiso'
        // (ISO user-data) namespace it's a 4-byte user-data FourCC, not text.
        let keyd = Keyd {
            key_namespace: FourCC::new(b"uiso"),
            key_value: b"cprt".to_vec(),
        };

        let mut buf = Vec::new();
        keyd.encode(&mut buf).unwrap();

        let decoded = Keyd::decode(&mut buf.as_slice()).expect("failed to decode keyd");
        assert_eq!(decoded, keyd);
    }

    #[test]
    fn test_loca_roundtrip() {
        let loca = Loca {
            locale: "fr-FR".into(),
        };

        let mut buf = Vec::new();
        loca.encode(&mut buf).unwrap();

        let decoded = Loca::decode(&mut buf.as_slice()).expect("failed to decode loca");
        assert_eq!(decoded, loca);
    }

    #[test]
    fn test_dtyp_well_known_roundtrip() {
        let dtyp = Dtyp {
            data_type: DataType::WellKnown(23), // BE float32
        };

        let mut buf = Vec::new();
        dtyp.encode(&mut buf).unwrap();

        let decoded = Dtyp::decode(&mut buf.as_slice()).expect("failed to decode dtyp");
        assert_eq!(decoded, dtyp);
    }

    #[test]
    fn test_dtyp_custom_roundtrip() {
        // The custom-namespace `datatype array` has no null terminator; it
        // fills the rest of the atom.
        let dtyp = Dtyp {
            data_type: DataType::Custom("com.example.custom-type".into()),
        };

        let mut buf = Vec::new();
        dtyp.encode(&mut buf).unwrap();

        let decoded = Dtyp::decode(&mut buf.as_slice()).expect("failed to decode dtyp");
        assert_eq!(decoded, dtyp);
    }

    #[test]
    fn test_dtyp_unknown_namespace_preserves_bytes() {
        // A `datatype_namespace` other than 0/1 should still round-trip its
        // raw bytes unchanged, per spec ("should be ignored" but "some
        // processing is still possible... such as copying it between tracks").
        let dtyp = Dtyp {
            data_type: DataType::Unknown(99, vec![1, 2, 3, 4]),
        };

        let mut buf = Vec::new();
        dtyp.encode(&mut buf).unwrap();

        let decoded = Dtyp::decode(&mut buf.as_slice()).expect("failed to decode dtyp");
        assert_eq!(decoded, dtyp);
    }
}
