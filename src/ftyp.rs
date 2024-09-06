use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ftyp {
    pub major_brand: FourCC,
    pub minor_version: u32,
    pub compatible_brands: Vec<FourCC>,
}

impl Atom for Ftyp {
    const KIND: FourCC = FourCC::new(b"ftyp");

    fn decode_body(buf: &mut Bytes) -> Result<Self> {
        Ok(Ftyp {
            major_brand: buf.decode()?,
            minor_version: buf.decode()?,
            compatible_brands: buf.decode()?,
        })
    }

    fn encode_body(&self, buf: &mut BytesMut) -> Result<()> {
        self.major_brand.encode(buf)?;
        self.minor_version.encode(buf)?;
        self.compatible_brands.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ftyp() {
        let decoded = Ftyp {
            major_brand: b"isom".into(),
            minor_version: 0,
            compatible_brands: vec![
                b"isom".into(),
                b"isom".into(),
                b"iso2".into(),
                b"avc1".into(),
                b"mp41".into(),
            ],
        };

        let mut buf = BytesMut::new();
        decoded.encode(&mut buf).expect("failed to encode");

        let mut buf = buf.freeze();
        let result = Ftyp::decode(&mut buf).expect("failed to decode");
        assert_eq!(decoded, result);
    }
}
