use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Smhd {
    pub balance: FixedPoint<i8>,
}

impl AtomExt for Smhd {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"smhd");

    fn decode_atom_ext(buf: &mut Bytes, _ext: ()) -> Result<Self> {
        let balance = buf.decode()?;
        u16::decode(buf)?; // reserved?

        Ok(Smhd { balance })
    }

    fn encode_atom_ext(&self, buf: &mut BytesMut) -> Result<()> {
        self.balance.encode(buf)?;
        0u16.encode(buf)?; // reserved

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smhd() {
        let expected = Smhd {
            balance: FixedPoint::from(-1),
        };
        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let decoded = Smhd::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
