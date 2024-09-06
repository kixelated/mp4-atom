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
    const KIND_EXT: FourCC = FourCC::new(b"mfhd");

    fn decode_body_ext(buf: &mut Bytes, _ext: ()) -> Result<Self> {
        Ok(Mfhd {
            sequence_number: buf.decode()?,
        })
    }

    fn encode_body_ext(&self, buf: &mut BytesMut) -> Result<()> {
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
        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let decoded = Mfhd::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
