use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Hdlr {
    pub handler_type: FourCC,
    pub name: String,
}

impl AtomExt for Hdlr {
    type Ext = ();
    const KIND: FourCC = FourCC::new(b"hdlr");

    fn decode_atom(buf: &mut Buf, _ext: ()) -> Result<Self> {
        u32::decode(buf)?; // pre-defined
        let handler = u32::decode(buf)?;

        buf.skip(12)?; // reserved

        let name = String::decode(buf)?;

        Ok(Hdlr {
            handler_type: From::from(handler),
            name,
        })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        buf.u32(0)?; // pre-defined
        self.handler_type.encode(buf)?;

        // 12 bytes reserved
        buf.zero(12)?;

        self.name.encode(buf)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hdlr() {
        let expected = Hdlr {
            handler_type: str::parse::<FourCC>("vide").unwrap(),
            name: String::from("VideoHandler"),
        };
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Hdlr::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_hdlr_empty() {
        let expected = Hdlr {
            handler_type: str::parse::<FourCC>("vide").unwrap(),
            name: String::new(),
        };
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Hdlr::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
