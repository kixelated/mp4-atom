use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Stsc {
    pub entries: Vec<StscEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StscEntry {
    pub first_chunk: u32,
    pub samples_per_chunk: u32,
    pub sample_description_index: u32,
    pub first_sample: u32,
}

impl AtomExt for Stsc {
    type Ext = ();

    const KIND: FourCC = FourCC::new(b"stsc");

    fn decode_atom(buf: &mut Buf, _ext: ()) -> Result<Self> {
        let entry_count = u32::decode(buf)?;

        let mut entries = Vec::new();
        for _ in 0..entry_count {
            let entry = StscEntry {
                first_chunk: u32::decode(buf)?,
                samples_per_chunk: u32::decode(buf)?,
                sample_description_index: u32::decode(buf)?,
                first_sample: 0,
            };
            entries.push(entry);
        }

        let mut sample_id = 1;
        for i in 0..entry_count {
            let (first_chunk, samples_per_chunk) = {
                let entry = entries.get_mut(i as usize).unwrap();
                entry.first_sample = sample_id;
                (entry.first_chunk, entry.samples_per_chunk)
            };
            if i < entry_count - 1 {
                let next_entry = entries.get(i as usize + 1).unwrap();
                sample_id = next_entry
                    .first_chunk
                    .checked_sub(first_chunk)
                    .and_then(|n| n.checked_mul(samples_per_chunk))
                    .and_then(|n| n.checked_add(sample_id))
                    .ok_or(Error::InvalidData(
                        "attempt to calculate stsc sample_id with overflow",
                    ))?;
            }
        }

        Ok(Stsc { entries })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        buf.u32(self.entries.len() as u32)?;
        for entry in self.entries.iter() {
            entry.first_chunk.encode(buf)?;
            entry.samples_per_chunk.encode(buf)?;
            entry.sample_description_index.encode(buf)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stsc() {
        let expected = Stsc {
            entries: vec![
                StscEntry {
                    first_chunk: 1,
                    samples_per_chunk: 1,
                    sample_description_index: 1,
                    first_sample: 1,
                },
                StscEntry {
                    first_chunk: 19026,
                    samples_per_chunk: 14,
                    sample_description_index: 1,
                    first_sample: 19026,
                },
            ],
        };
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Stsc::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
