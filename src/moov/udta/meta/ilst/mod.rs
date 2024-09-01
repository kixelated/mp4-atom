mod covr;
mod desc;
mod name;
mod year;

pub use covr::*;
pub use desc::*;
pub use name::*;
pub use year::*;


#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ilst {
    pub name: Option<Name>,
    pub year: Option<Year>, // Called day in the spec
    pub covr: Option<Covr>,
    pub desc: Option<Desc>,
}

impl Atom for Ilst {
    const KIND: FourCC = FourCC::new(b"ilst");

    fn decode_atom(buf: &mut Bytes) -> Result<Self> {
        let mut name = None;
        let mut year = None;
        let mut covr = None;
        let mut desc = None;

        while let Some(atom) = buf.decode()? {
            match atom {
                Any::Name(atom) => name = atom.into(),
                Any::Year(atom) => year = atom.into(),
                Any::Covr(atom) => covr = atom.into(),
                Any::Desc(atom) => desc = atom.into(),
                Any::Unknown(kind, _) => tracing::warn!("unknown atom: {:?}", kind),
                atom => return Err(Error::UnexpectedBox(atom.kind())),
            }
        }

        Ok(Ilst {
            name,
            year,
            covr,
            desc,
        })
    }

    fn encode_atom(&self, buf: &mut BytesMut) -> Result<()> {
        self.name.encode(buf)?;
        self.year.encode(buf)?;
        self.covr.encode(buf)?;
        self.desc.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_ilst() {
        let expected = Ilst {
            year: Year("src_year".to_string()).into(),
            ..Default::default()
        };

        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let decoded = Ilst::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_ilst_empty() {
        let expected = Ilst::default();
        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let decoded = Ilst::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
