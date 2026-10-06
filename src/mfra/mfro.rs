use crate::*;

/// MovieFragmentRandomAccessOffsetBox (`mfro`).
///
/// See ISO/IEC 14496-12:2022 section 8.8.11
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mfro {
    pub parent_size: u32,
}

impl AtomExt for Mfro {
    type Ext = ();
    const KIND_EXT: FourCC = FourCC::new(b"mfro");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        Ok(Mfro {
            parent_size: u32::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.parent_size.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mfro() {
        let expected = Mfro { parent_size: 96 };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Mfro::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
