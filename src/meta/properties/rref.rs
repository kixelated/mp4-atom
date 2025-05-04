use crate::*;

// RequiredReferenceTypesProperty, ISO/IEC 23008-12 Section 6.5.17
// Required references for a predictively encoded image item

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rref {
    pub reference_types: Vec<FourCC>,
}

impl AtomExt for Rref {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"rref");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let reference_type_count = u8::decode(buf)?;
        let mut reference_types = Vec::with_capacity(reference_type_count as usize);
        for _ in 0..reference_type_count {
            reference_types.push(FourCC::decode(buf)?);
        }
        Ok(Rref { reference_types })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let reference_type_count = self.reference_types.len() as u8;
        reference_type_count.encode(buf)?;
        for reference_type in &self.reference_types {
            reference_type.encode(buf)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rref() {
        let expected = Rref {
            reference_types: vec![FourCC::new(b"dpnd"), FourCC::new(b"base")],
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        assert_eq!(
            buf,
            [
                0, 0, 0, 0x15, b'r', b'r', b'e', b'f', 0, 0, 0, 0, 2, b'd', b'p', b'n', b'd', b'b',
                b'a', b's', b'e'
            ]
        );
        let decoded = Rref::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
