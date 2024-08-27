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

    const KIND: FourCC = FourCC::new(b"ctts");

    fn decode_atom(buf: &mut Buf, _ext: ()) -> Result<Self> {
        let entry_count = u32::decode(buf)?;

        let mut entries = Vec::new();
        for _ in 0..entry_count {
            let entry = CttsEntry {
                sample_count: u32::decode(buf)?,
                sample_offset: buf.i32()?,
            };
            entries.push(entry);
        }

        Ok(Ctts { entries })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        buf.u32(self.entries.len() as u32)?;
        for entry in self.entries.iter() {
            buf.u32(entry.sample_count)?;
            buf.i32(entry.sample_offset)?;
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
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Ctts::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
