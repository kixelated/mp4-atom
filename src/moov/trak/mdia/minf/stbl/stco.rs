use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Stco {
    pub entries: Vec<u32>,
}

impl AtomExt for Stco {
    type Ext = ();

    const KIND: FourCC = FourCC::new(b"stco");

    fn decode_atom(buf: &mut Buf, _ext: ()) -> Result<Self> {
        let count = u32::decode(buf)?;
        let mut entries = Vec::new();

        for _ in 0..count {
            let chunk_offset = u32::decode(buf)?;
            entries.push(chunk_offset?);
        }

        Ok(Stco { entries })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        buf.u32(self.entries.len() as u32)?;
        for chunk_offset in self.entries.iter() {
            buf.u32(chunk_offset)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stco() {
        let expected = Stco {
            entries: vec![267, 1970, 2535, 2803, 11843, 22223, 33584],
        };
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Stco::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
