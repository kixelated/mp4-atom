use crate::*;

// ImageScaling, ISO/IEC 23008-12 Section 6.5.13
// output width and height

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Iscl {
    pub target_width_numerator: u16,
    pub target_width_denominator: u16,
    pub target_height_numerator: u16,
    pub target_height_denominator: u16,
}

impl AtomExt for Iscl {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"iscl");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let target_width_numerator = u16::decode(buf)?;
        let target_width_denominator = u16::decode(buf)?;
        let target_height_numerator = u16::decode(buf)?;
        let target_height_denominator = u16::decode(buf)?;
        Ok(Iscl {
            target_width_numerator,
            target_width_denominator,
            target_height_numerator,
            target_height_denominator,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.target_width_numerator.encode(buf)?;
        self.target_width_denominator.encode(buf)?;
        self.target_height_numerator.encode(buf)?;
        self.target_height_denominator.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iscl() {
        let expected = Iscl {
            target_width_numerator: 10,
            target_width_denominator: 3,
            target_height_numerator: 20,
            target_height_denominator: 5,
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        assert_eq!(
            buf,
            [0, 0, 0, 20, b'i', b's', b'c', b'l', 0, 0, 0, 0, 0, 0x0a, 0, 0x03, 0, 0x14, 0, 0x05]
        );
        let decoded = Iscl::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
