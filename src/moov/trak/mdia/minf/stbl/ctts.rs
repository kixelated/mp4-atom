use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ctts {
    pub entries: Vec<CttsEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CttsEntry {
    pub sample_count: u32,
    pub sample_offset: i32,
}

impl AtomExt for Ctts {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"ctts");

    fn decode_atom_ext(buf: &mut Bytes, _ext: ()) -> Result<Self> {
        let entry_count = u32::decode(buf)?;

        let mut entries = Vec::new();
        for _ in 0..entry_count {
            let entry = CttsEntry {
                sample_count: u32::decode(buf)?,
                sample_offset: i32::decode(buf)?,
            };
            entries.push(entry);
        }

        Ok(Ctts { entries })
    }

    fn encode_atom_ext(&self, buf: &mut BytesMut) -> Result<()> {
        (self.entries.len() as u32).encode(buf)?;
        for entry in self.entries.iter() {
            (entry.sample_count).encode(buf)?;
            (entry.sample_offset).encode(buf)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_ctts() {
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
        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let decoded = Ctts::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
