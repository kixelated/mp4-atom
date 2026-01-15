use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sthd {}

impl AtomExt for Sthd {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"sthd");

    fn decode_body_ext<B: Buf>(_buf: &mut B, _ext: ()) -> Result<Self> {
        Ok(Sthd {})
    }

    fn encode_body_ext<B: BufMut>(&self, _buf: &mut B) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sthd() {
        let sthd = Sthd {};
        let mut buf = Vec::new();
        sthd.encode(&mut buf).unwrap();
        assert_eq!(
            buf,
            vec![0x00, 0x00, 0x00, 0x0c, b's', b't', b'h', b'd', 0x00, 0x00, 0x00, 0x00]
        );

        let mut buf = buf.as_ref();
        let decoded = Sthd::decode(&mut buf).unwrap();
        assert_eq!(decoded, sthd);
    }
}
