mod meta;
pub use meta::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Udta {
    pub meta: Option<Meta>,
}

impl Atom for Udta {
    const KIND: FourCC = FourCC::new(b"udta");

    fn decode_atom(buf: &mut Bytes) -> Result<Self> {
        let mut meta = buf.decode()?;
        Ok(Udta { meta })
    }

    fn encode_atom(&self, buf: &mut BytesMut) -> Result<()> {
        self.meta.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_udta_empty() {
        let expected = Udta { meta: None };

        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let output = Udta::decode(&mut buf).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_udta() {
        let expected = Udta {
            meta: Some(Meta::default()),
        };

        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let output = Udta::decode(&mut buf).unwrap();
        assert_eq!(output, expected);
    }
}
