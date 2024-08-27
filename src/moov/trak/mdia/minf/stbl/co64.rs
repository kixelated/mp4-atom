use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Co64 {
    pub entries: Vec<u64>,
}

impl AtomExt for Co64 {
    type Ext = ();

    const KIND: FourCC = FourCC::new(b"co64");

    fn decode_atom(buf: &mut Buf, _ext: ()) -> Result<Self> {
        let entry_count = u32::decode(buf)?;
        let mut entries = Vec::new();
        for _ in 0..entry_count {
            let chunk_offset = u64::decode(buf)?;
            entries.push(chunk_offset);
        }

        Ok(Co64 { entries })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        buf.u32(self.entries.len() as u32)?;
        for chunk_offset in self.entries.iter() {
            buf.u64(chunk_offset)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_co64() {
        let expected = Co64 {
            entries: vec![267, 1970, 2535, 2803, 11843, 22223, 33584],
        };
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Co64::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
