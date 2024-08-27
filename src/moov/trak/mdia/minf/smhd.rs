use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Smhd {
    pub balance: Ratio<i8>,
}

impl AtomExt for Smhd {
    type Ext = ();

    const KIND: FourCC = FourCC::new(b"smhd");

    fn decode_atom(buf: &mut Buf, _ext: ()) -> Result<Self> {
        let balance = buf.decode()?;
        buf.u16()?; // reserved?

        Ok(Smhd { balance })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        self.balance.encode(buf)?;
        buf.u16(0)?; // reserved

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smhd() {
        let expected = Smhd { balance: -1.into() };
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Smhd::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
