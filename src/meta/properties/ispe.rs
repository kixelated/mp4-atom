use crate::*;

// ImageSpatialExtentProperty, ISO/IEC 23008-12 Section 6.5.3
// Height and width of the image.

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ispe {
    pub width: u32,
    pub height: u32,
}

impl AtomExt for Ispe {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"ispe");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let width = u32::decode(buf)?;
        let height = u32::decode(buf)?;
        Ok(Ispe { width, height })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.width.encode(buf)?;
        self.height.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ispe() {
        let expected = Ispe {
            width: 1024,
            height: 768,
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        assert_eq!(
            buf,
            [0, 0, 0, 20, b'i', b's', b'p', b'e', 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 3, 0]
        );
        let decoded = Ispe::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
