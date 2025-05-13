use crate::*;

// ImageRotation, ISO/IEC 23008-12 Section 6.5.10
// Image rotation operation (transformative property).

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Irot {
    pub angle: u8,
}

impl Atom for Irot {
    const KIND: FourCC = FourCC::new(b"irot");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Irot {
            angle: u8::decode(buf)? & 0x03,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.angle.encode(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_irot() {
        let expected = Irot { angle: 3 };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        assert_eq!(buf, [0, 0, 0, 9, b'i', b'r', b'o', b't', 3]);
        let decoded = Irot::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
