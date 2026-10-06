use crate::*;

ext! {
    name: Ctts,
    versions: [0, 1],
    flags: {}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ctts {
    pub entries: Vec<CttsEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CttsEntry {
    pub sample_count: u32,
    /// Unsigned for version 0 boxes and signed for version 1 boxes.
    pub sample_offset: i64,
}

impl AtomExt for Ctts {
    type Ext = CttsExt;

    const KIND_EXT: FourCC = FourCC::new(b"ctts");

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: CttsExt) -> Result<Self> {
        let entry_count = u32::decode(buf)?;

        let mut entries = Vec::new();
        for _ in 0..entry_count {
            let entry = CttsEntry {
                sample_count: u32::decode(buf)?,
                sample_offset: match ext.version {
                    CttsVersion::V0 => u32::decode(buf)?.into(),
                    CttsVersion::V1 => i32::decode(buf)?.into(),
                },
            };
            entries.push(entry);
        }

        Ok(Ctts { entries })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<CttsExt> {
        let version = if self.entries.iter().any(|entry| entry.sample_offset < 0) {
            CttsVersion::V1
        } else {
            CttsVersion::V0
        };

        (self.entries.len() as u32).encode(buf)?;
        for entry in self.entries.iter() {
            (entry.sample_count).encode(buf)?;
            match version {
                CttsVersion::V0 => u32::try_from(entry.sample_offset)
                    .map_err(|_| Error::InvalidSize)?
                    .encode(buf)?,
                CttsVersion::V1 => i32::try_from(entry.sample_offset)
                    .map_err(|_| Error::InvalidSize)?
                    .encode(buf)?,
            }
        }

        Ok(version.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctts_v1() {
        const ENCODED: &[u8] = &[
            0x00, 0x00, 0x00, 0x20, b'c', b't', b't', b's', 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xc8, 0x00, 0x00, 0x00, 0x02,
            0xff, 0xff, 0xff, 0x9c,
        ];

        let expected = Ctts {
            entries: vec![
                CttsEntry {
                    sample_count: 1,
                    sample_offset: 200,
                },
                CttsEntry {
                    sample_count: 2,
                    sample_offset: -100,
                },
            ],
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();
        assert_eq!(buf, ENCODED);

        let mut buf = buf.as_ref();
        let decoded = Ctts::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_ctts_v0_unsigned_offset() {
        const ENCODED: &[u8] = &[
            0x00, 0x00, 0x00, 0x18, b'c', b't', b't', b's', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x80, 0x00, 0x00, 0x00,
        ];

        let decoded = Ctts::decode(&mut &ENCODED[..]).unwrap();
        assert_eq!(
            decoded,
            Ctts {
                entries: vec![CttsEntry {
                    sample_count: 2,
                    sample_offset: 1 << 31,
                }],
            }
        );

        let mut encoded = Vec::new();
        decoded.encode(&mut encoded).unwrap();
        assert_eq!(encoded, ENCODED);
    }
}
