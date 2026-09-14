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

        let count = u8::decode(buf)?;
        let mut entries = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let start = u64::decode(buf)?;

            let title_len = u8::decode(buf)? as usize;
            if title_len > buf.remaining() {
                return Err(Error::OutOfBounds);
            }
            let title_bytes = buf.slice(title_len).to_vec();
            buf.advance(title_len);
            let title = String::from_utf8(title_bytes)
                .map_err(|err| Error::InvalidString(err.to_string()))?;

            entries.push(ChplEntry { start, title });
        }

        Ok(Self { entries })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        1u8.encode(buf)?; // version
        [0u8; 3].encode(buf)?; // flags
        [0u8; 4].encode(buf)?; // reserved

        let count: u8 = self
            .entries
            .len()
            .try_into()
            .map_err(|_| Error::TooLarge(Self::KIND))?;
        count.encode(buf)?;

        for entry in &self.entries {
            entry.start.encode(buf)?;

            let title_bytes = entry.title.as_bytes();
            let title_len: u8 = title_bytes
                .len()
                .try_into()
                .map_err(|_| Error::TooLarge(Self::KIND))?;
            title_len.encode(buf)?;
            buf.append_slice(title_bytes);
        }

        Ok(())
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
