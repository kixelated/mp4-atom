use crate::*;

/// One chapter entry inside a [`Chpl`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChplEntry {
    /// Start time, in 100ns units, relative to the start of the presentation.
    pub start: u64,
    pub title: String,
}

/// Nero-style Chapter List Box (`chpl`).
///
/// Not part of the ISO/QuickTime spec, but a de facto standard written and read
/// by e.g. Nero, `ffmpeg`'s `mov` muxer/demuxer, and MP4Box, as an alternative
/// to the QuickTime `chap` track reference + text track convention.
///
/// Version 0 and version 1 only differ in an extra 4-byte reserved field
/// present after the FullBox header in version 1 (absent in version 0); it
/// carries no known meaning in either case, so it's not exposed here. This
/// always encodes as version 1, matching the common convention used by
/// `ffmpeg`'s `mov` muxer and MP4Box.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Chpl {
    pub entries: Vec<ChplEntry>,
}

impl Atom for Chpl {
    const KIND: FourCC = FourCC::new(b"chpl");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let version = u8::decode(buf)?;
        <[u8; 3]>::decode(buf)?; // flags
        if version != 0 {
            <[u8; 4]>::decode(buf)?; // reserved, version 1 only
        }

        let entries = buf.decode_list(64u8, |buf| {
            Ok(ChplEntry {
                start: u64::decode(buf)?,
                title: decode_length_prefixed_utf8_string::<u8>(buf)?,
            })
        })?;

        Ok(Self { entries })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        1u8.encode(buf)?; // version
        [0u8; 3].encode(buf)?; // flags
        [0u8; 4].encode(buf)?; // reserved

        buf.encode_list::<u8, _>(&self.entries, Self::KIND, |buf, entry| {
            entry.start.encode(buf)?;

            encode_length_prefixed_string::<u8>(buf, &entry.title, Self::KIND)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chpl_roundtrip() {
        let expected = Chpl {
            entries: vec![
                ChplEntry {
                    start: 0,
                    title: "Intro".into(),
                },
                ChplEntry {
                    start: 30_000_000,
                    title: "Chapter 2".into(),
                },
            ],
        };

        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let decoded = Chpl::decode(&mut buf.as_slice()).expect("failed to decode chpl");
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_chpl_empty() {
        let expected = Chpl { entries: vec![] };

        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let decoded = Chpl::decode(&mut buf.as_slice()).expect("failed to decode chpl");
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_chpl_version0_no_reserved_field() {
        // Per ffmpeg's mov.c (mov_read_chpl): version 0 omits the 4-byte
        // reserved field that version 1 has after the FullBox header.
        let expected = Chpl {
            entries: vec![
                ChplEntry {
                    start: 0,
                    title: "Intro".into(),
                },
                ChplEntry {
                    start: 30_000_000,
                    title: "Chapter 2".into(),
                },
            ],
        };

        let mut buf = Vec::new();
        0u8.encode(&mut buf).unwrap(); // version 0
        [0u8; 3].encode(&mut buf).unwrap(); // flags
        let count: u8 = expected.entries.len().try_into().unwrap();
        count.encode(&mut buf).unwrap();
        for entry in &expected.entries {
            entry.start.encode(&mut buf).unwrap();
            let title_bytes = entry.title.as_bytes();
            let title_len: u8 = title_bytes.len().try_into().unwrap();
            title_len.encode(&mut buf).unwrap();
            buf.extend_from_slice(title_bytes);
        }

        let decoded = Chpl::decode_body(&mut buf.as_slice()).expect("failed to decode chpl");
        assert_eq!(decoded, expected);
    }
}
