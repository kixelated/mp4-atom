mod meta;
pub use meta::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Udta {
    pub meta: Option<Meta>,
}

impl Atom for Udta {
    const KIND: FourCC = FourCC::new(b"udta");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let meta = Option::decode(buf)?;
        Ok(Udta { meta })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
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

        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let output = Udta::decode(&mut buf).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_udta() {
        let expected = Udta {
            meta: Some(Meta::default()),
        };

        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let output = Udta::decode(&mut buf).unwrap();
        assert_eq!(output, expected);
    }
}
