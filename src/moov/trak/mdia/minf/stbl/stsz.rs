use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Stsz {
    // TODO maybe use an enum to be more efficient
    pub sample_sizes: Vec<u32>,
}

impl AtomExt for Stsz {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"stsz");

    fn decode_body_ext(buf: &mut Bytes, _ext: ()) -> Result<Self> {
        let sample_size = u32::decode(buf)?;
        let sample_count = u32::decode(buf)?;

        let mut sample_sizes = Vec::new();
        if sample_size == 0 {
            for _ in 0..sample_count {
                let sample_number = u32::decode(buf)?;
                sample_sizes.push(sample_number);
            }
        } else {
            sample_sizes = vec![sample_size; sample_count as usize];
        }

        Ok(Stsz { sample_sizes })
    }

    fn encode_body_ext(&self, buf: &mut BytesMut) -> Result<()> {
        if self.sample_sizes.is_empty() {
            0u32.encode(buf)?;
            0u32.encode(buf)?;
            return Ok(());
        }

        let same = self.sample_sizes.iter().all(|&x| x == self.sample_sizes[0]);
        if same {
            self.sample_sizes[0].encode(buf)?;
            (self.sample_sizes.len() as u32).encode(buf)?;
        } else {
            0u32.encode(buf)?;
            (self.sample_sizes.len() as u32).encode(buf)?;

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
            sample_sizes: vec![1165, 1165, 1165, 1165],
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
            sample_sizes: vec![1165, 11, 11, 8545, 10126, 10866, 9643, 9351, 7730],
        };
        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let decoded = Stsz::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
