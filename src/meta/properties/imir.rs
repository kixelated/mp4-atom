use crate::*;

// ImageMirror, ISO/IEC 23008-12 Section 6.5.12
// Image mirror operation (transformative property).

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Imir {
    pub axis: u8,
}

impl Atom for Imir {
    const KIND: FourCC = FourCC::new(b"imir");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Imir {
            axis: u8::decode(buf)?,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.axis.encode(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ispe() {
        let expected = Imir { axis: 1 };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        assert_eq!(buf, [0, 0, 0, 9, b'i', b'm', b'i', b'r', 1]);
        let decoded = Imir::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
