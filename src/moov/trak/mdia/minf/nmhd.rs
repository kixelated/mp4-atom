use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Nmhd {}

impl AtomExt for Nmhd {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"nmhd");

    fn decode_body_ext<B: Buf>(_buf: &mut B, _ext: ()) -> Result<Self> {
        Ok(Nmhd {})
    }

    fn encode_body_ext<B: BufMut>(&self, _buf: &mut B) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nmhd() {
        let nmhd = Nmhd {};
        let mut buf = Vec::new();
        nmhd.encode(&mut buf).unwrap();
        assert_eq!(
            buf,
            vec![0x00, 0x00, 0x00, 0x0c, 0x6e, 0x6d, 0x68, 0x64, 0x00, 0x00, 0x00, 0x00]
        );

        let mut buf = buf.as_ref();
        let decoded = Nmhd::decode(&mut buf).unwrap();
        assert_eq!(decoded, nmhd);
    }
}
