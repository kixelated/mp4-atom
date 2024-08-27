use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Stsz {
    pub sample_size: u32,
    pub sample_count: u32,
    pub sample_sizes: Vec<u32>,
}

impl AtomExt for Stsz {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"stsz");

    fn decode_atom_ext(buf: &mut Bytes, _ext: ()) -> Result<Self> {
        let sample_size = u32::decode(buf)?;
        let sample_count = u32::decode(buf)?;

        let mut sample_sizes = Vec::new();
        if sample_size == 0 {
            for _ in 0..sample_count {
                let sample_number = u32::decode(buf)?;
                sample_sizes.push(sample_number);
            }
        }

        Ok(Stsz {
            sample_size,
            sample_count,
            sample_sizes,
        })
    }

    fn encode_atom_ext(&self, buf: &mut BytesMut) -> Result<()> {
        self.sample_size.encode(buf)?;
        self.sample_count.encode(buf)?;

        if self.sample_size == 0 {
            if self.sample_count != self.sample_sizes.len() as u32 {
                return Err(Error::InvalidData("sample count out of sync"));
            }
            for sample_number in self.sample_sizes.iter() {
                sample_number.encode(buf)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stsz_same_size() {
        let expected = Stsz {
            sample_size: 1165,
            sample_count: 12,
            sample_sizes: vec![],
        };
        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let decoded = Stsz::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_stsz_many_sizes() {
        let expected = Stsz {
            sample_size: 0,
            sample_count: 9,
            sample_sizes: vec![1165, 11, 11, 8545, 10126, 10866, 9643, 9351, 7730],
        };
        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let decoded = Stsz::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
