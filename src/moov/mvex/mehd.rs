use crate::*;

ext! {
    name: Mehd,
    versions: [0,1],
    flags: {}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Mehd {
    pub fragment_duration: u64,
}

impl AtomExt for Mehd {
    const KIND: FourCC = FourCC::new(b"mehd");

    type Ext = MehdExt;

    fn decode_atom(buf: &mut Buf, ext: MehdExt) -> Result<Self> {
        let fragment_duration = match ext.version {
            MehdVersion::V1 => u64::decode(buf)?,
            MehdVersion::V0 => u32::decode(buf)? as u64,
        };

        Ok(Mehd { fragment_duration })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<MehdExt> {
        self.fragment_duration.encode(buf)?;
        Ok(MehdVersion::V1.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mehd32() {
        let expected = Mehd {
            fragment_duration: 32,
        };
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Mehd::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_mehd64() {
        let expected = Mehd {
            fragment_duration: 30439936,
        };
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Mehd::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
