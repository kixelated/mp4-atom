use std::borrow::Borrow;

use crate::*;

/// One item in a [`MebxSample`]: a value box whose type is a `local_key_id`
/// declared in the corresponding [`Mebx`] sample entry's `keys` table.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MebxItem {
    pub local_key_id: FourCC,
    pub data: Vec<u8>,
}

/// The content of a single sample in a `mebx` (multiplexed/"boxed" timed
/// metadata) track (the [`Mebx`] sample entry): a concatenation of one or
/// more value boxes, one per declared key present at this sample's time.
///
/// Sample content lives inside `mdat` and isn't walked automatically by this
/// crate -- decode it explicitly via [`MebxSample::decode`]. Unlike
/// [`EvteSample`](crate::EvteSample), a value box's own type isn't drawn from
/// a fixed set: it's the arbitrary `local_key_id` the muxer chose per key
/// (see [`MebxKey`]), so this just preserves each item's raw type and bytes
/// rather than interpreting them.
///
/// See ISO/IEC 14496-12:2026 Section 12.8.3.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MebxSample;

impl MebxSample {
    pub fn decode<B: Buf>(buf: &mut B) -> Result<Vec<MebxItem>> {
        let mut items = Vec::new();
        let mut found_null = false;

        while let Some(header) = Header::decode_maybe(buf)? {
            let size = header.size.unwrap_or(buf.remaining());
            if size > buf.remaining() {
                return Err(Error::OutOfBounds);
            }
            let body = buf.slice(size);

            if u32::from(header.kind) == 0 {
                found_null = true;
            } else {
                items.push(MebxItem {
                    local_key_id: header.kind,
                    data: body.to_vec(),
                });
            }

            buf.advance(size);
        }

        if buf.has_remaining() {
            Err(Error::ShortRead)
        } else if items.is_empty() && !found_null {
            Err(Error::UnexpectedEof)
        } else {
            Ok(items)
        }
    }

    pub fn encode<B: BufMut>(
        items: impl IntoIterator<Item = impl Borrow<MebxItem>>,
        buf: &mut B,
    ) -> Result<()> {
        let mut wrote_atom = false;

        for item in items {
            let item = item.borrow();
            buf.encode_atom(item.local_key_id, |buf| {
                buf.append_slice(&item.data);
                Ok(())
            })?;
            wrote_atom = true;
        }

        if wrote_atom {
            Ok(())
        } else {
            buf.encode_atom(0u32, |_| Ok(()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mebx_sample_empty_roundtrip() {
        let sample = vec![];

        let mut buf = Vec::new();
        MebxSample::encode(&sample, &mut buf).unwrap();
        assert_eq!(buf, b"\0\0\0\x08\0\0\0\0");

        let decoded = MebxSample::decode(&mut buf.as_slice()).expect("failed to decode sample");
        assert_eq!(decoded, sample);
    }

    #[test]
    fn test_mebx_sample_roundtrip() {
        let sample = vec![
            MebxItem {
                local_key_id: FourCC::new(b"test"),
                data: b"example".to_vec(),
            },
            MebxItem {
                local_key_id: FourCC::new(b"othr"),
                data: b"something".to_vec(),
            },
        ];

        let mut buf = Vec::new();
        MebxSample::encode(&sample, &mut buf).unwrap();

        let decoded = MebxSample::decode(&mut buf.as_slice()).expect("failed to decode sample");
        assert_eq!(decoded, sample);
    }

    #[test]
    fn test_mebx_sample_ignore_null_items() {
        // `local_key_id` 0 signals "no metadata of this kind at this time"
        // and may be used as padding; its content is ignored, not rejected.
        let mut buf = Vec::new();
        buf.encode_atom(0u32, |buf| [1, 2].encode(buf)).unwrap();
        buf.encode_atom(b"test", |buf| [3, 4].encode(buf)).unwrap();
        buf.encode_atom(0u32, |_| Ok(())).unwrap();

        let decoded = MebxSample::decode(&mut buf.as_slice()).expect("failed to decode sample");
        assert_eq!(
            decoded,
            [MebxItem {
                local_key_id: FourCC::new(b"test"),
                data: vec![3, 4],
            }]
        );
    }

    #[test]
    fn test_mebx_sample_empty_is_error() {
        let err = MebxSample::decode(&mut [].as_slice()).unwrap_err();
        assert!(matches!(err, Error::UnexpectedEof));
    }

    #[test]
    fn test_mebx_sample_short_read() {
        let err = MebxSample::decode(&mut [1, 2].as_slice()).unwrap_err();
        assert!(matches!(err, Error::ShortRead));
    }
}
