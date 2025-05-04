use crate::*;

// AuxiliaryTypeProperty, ISO/IEC 23008-12 Section 6.5.8
// Describes auxiliary images (e.g. alpha planes).

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Auxc {
    pub aux_type: String,
    pub aux_subtype: Vec<u8>,
}

impl AtomExt for Auxc {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"auxC");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let aux_type = String::decode(buf)?;
        let aux_subtype = Vec::decode(buf)?;
        Ok(Auxc {
            aux_type,
            aux_subtype,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.aux_type.as_str().encode(buf)?;
        self.aux_subtype.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn test_auxc() {
        let expected = Auxc {
            aux_type: "something".to_string(),
            aux_subtype: vec![3, 2, 1],
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Auxc::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
