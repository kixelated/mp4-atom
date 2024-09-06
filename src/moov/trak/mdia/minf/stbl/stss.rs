use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Stss {
    pub entries: Vec<u32>,
}

impl AtomExt for Stss {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"stss");

    fn decode_body_ext(buf: &mut Bytes, _ext: ()) -> Result<Self> {
        let entry_count = u32::decode(buf)?;

        let mut entries = Vec::new();
        for _ in 0..entry_count {
            let sample_number = u32::decode(buf)?;
            entries.push(sample_number);
        }

        Ok(Stss { entries })
    }

    fn encode_body_ext(&self, buf: &mut BytesMut) -> Result<()> {
        (self.entries.len() as u32).encode(buf)?;
        for sample_number in self.entries.iter() {
            sample_number.encode(buf)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stss() {
        let expected = Stss {
            entries: vec![1, 61, 121, 181, 241, 301, 361, 421, 481],
        };
        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let decoded = Stss::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
