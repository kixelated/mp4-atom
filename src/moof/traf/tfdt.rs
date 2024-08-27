use crate::*;

ext! {
    name: Tfdt,
    versions: [0, 1],
    flags: {}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tfdt {
    pub base_media_decode_time: u64,
}

impl AtomExt for Tfdt {
    const KIND: FourCC = FourCC::new(b"tfdt");

    type Ext = TfdtExt;

    fn decode_atom(buf: &mut Buf, ext: TfdtExt) -> Result<Self> {
        let base_media_decode_time = match ext.version {
            TfdtVersion::V1 => u64::decode(buf)?,
            TfdtVersion::V0 => u32::decode(buf)? as u64,
        };

        Ok(Tfdt {
            base_media_decode_time,
        })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<TfdtExt> {
        self.base_media_decode_time.encode(buf)?;
        Ok(TfdtVersion::V1.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tfdt32() {
        let expected = Tfdt {
            base_media_decode_time: 0,
        };
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Tfdt::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_tfdt64() {
        let expected = Tfdt {
            base_media_decode_time: u32::MAX + 1,
        };

        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Tfdt::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
