use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mfhd {
    pub sequence_number: u32,
}

impl Default for Mfhd {
    fn default() -> Self {
        Mfhd { sequence_number: 1 }
    }
}

impl AtomExt for Mfhd {
    type Ext = ();
    const KIND: FourCC = FourCC::new(b"mfhd");

    fn decode_atom(buf: &mut Buf, _ext: ()) -> Result<Self> {
        Ok(Mfhd {
            sequence_number: u32::decode(buf)?,
        })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        self.sequence_number.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mfhd() {
        let expected = Mfhd { sequence_number: 1 };
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Mfhd::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
