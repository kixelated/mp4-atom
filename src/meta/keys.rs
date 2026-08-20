use crate::*;

/// A single entry in a Metadata Item Keys Box ('keys').
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeyEntry {
    pub key_namespace: FourCC,
    pub key_value: String,
}

/// Metadata Item Keys Box ('keys').
///
/// Used alongside a `hdlr` with `handler_type == 'mdta'` (Apple's key-based metadata
/// scheme): each `ilst` item under this scheme is keyed by a 1-based numeric index
/// into this table instead of a well-known FourCC, and this box gives that index a
/// namespaced string key (e.g. `com.apple.quicktime.make`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Keys {
    pub entries: Vec<KeyEntry>,
}

impl AtomExt for Keys {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"keys");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let entry_count = u32::decode(buf)?;
        let mut entries = Vec::with_capacity((entry_count as usize).min(4096));

        for _ in 0..entry_count {
            let header = Header::decode(buf)?;
            let size = header.size.ok_or(Error::InvalidSize)?;
            if size > buf.remaining() {
                return Err(Error::OutOfBounds);
            }

            let key_value = String::from_utf8(buf.slice(size).to_vec())
                .map_err(|err| Error::InvalidString(err.to_string()))?;
            buf.advance(size);

            entries.push(KeyEntry {
                key_namespace: header.kind,
                key_value,
            });
        }

        Ok(Self { entries })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        (self.entries.len() as u32).encode(buf)?;

        for entry in &self.entries {
            let header = Header {
                kind: entry.key_namespace,
                size: Some(entry.key_value.len()),
            };
            header.encode(buf)?;
            entry.key_value.as_bytes().encode(buf)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keys_roundtrip() {
        let keys = Keys {
            entries: vec![
                KeyEntry {
                    key_namespace: FourCC::new(b"mdta"),
                    key_value: "com.apple.quicktime.make".into(),
                },
                KeyEntry {
                    key_namespace: FourCC::new(b"mdta"),
                    key_value: "com.apple.quicktime.model".into(),
                },
            ],
        };

        let mut buf = Vec::new();
        keys.encode(&mut buf).unwrap();

        let decoded = Keys::decode(&mut buf.as_slice()).expect("failed to decode keys");
        assert_eq!(decoded, keys);
    }

    #[test]
    fn test_keys_empty() {
        let keys = Keys::default();

        let mut buf = Vec::new();
        keys.encode(&mut buf).unwrap();

        let decoded = Keys::decode(&mut buf.as_slice()).expect("failed to decode keys");
        assert_eq!(decoded, keys);
    }

    // A declared per-entry size larger than what's actually left in the box
    // must be a decode error, not a panic from an out-of-bounds slice.
    #[test]
    fn test_keys_truncated_entry() {
        let mut buf = Vec::new();
        0u32.encode(&mut buf).unwrap(); // version + flags
        1u32.encode(&mut buf).unwrap(); // entry_count
        100u32.encode(&mut buf).unwrap(); // header size (way beyond what follows)
        FourCC::new(b"mdta").encode(&mut buf).unwrap();
        buf.extend_from_slice(b"short");

        let err = Keys::decode_body(&mut buf.as_slice()).unwrap_err();
        assert!(matches!(err, Error::OutOfBounds));
    }

    // A bogus/huge entry_count with no matching entries must fail cleanly
    // once the buffer runs out, rather than allocate or loop unboundedly.
    #[test]
    fn test_keys_malformed_count() {
        let mut buf = Vec::new();
        0u32.encode(&mut buf).unwrap(); // version + flags
        u32::MAX.encode(&mut buf).unwrap(); // entry_count
        FourCC::new(b"mdta").encode(&mut buf).unwrap(); // truncated first header

        let err = Keys::decode_body(&mut buf.as_slice()).unwrap_err();
        assert!(matches!(err, Error::OutOfBounds));
    }
}
