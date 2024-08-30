use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hdlr {
    pub handler_type: FourCC,
    pub name: String,
}

impl Default for Hdlr {
    fn default() -> Self {
        Hdlr {
            handler_type: FourCC::new(b"none"),
            name: String::new(),
        }
    }
}

impl AtomExt for Hdlr {
    type Ext = ();
    const KIND_EXT: FourCC = FourCC::new(b"hdlr");

    fn decode_atom_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        u32::decode(buf)?; // pre-defined
        let handler = u32::decode(buf)?;

        <[u8; 12]>::decode(buf)?; // reserved

        let name = String::decode(buf)?;

        Ok(Hdlr {
            handler_type: From::from(handler),
            name,
        })
    }

    fn encode_atom_ext(&self, buf: &mut BytesMut) -> Result<()> {
        0u32.encode(buf)?; // pre-defined
        self.handler_type.encode(buf)?;

        // 12 bytes reserved
        [0u8; 12].encode(buf)?;

        self.name.as_str().encode(buf)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_hdlr() {
        let expected = Hdlr {
            handler_type: FourCC::new(b"vide"),
            name: String::from("VideoHandler"),
        };
        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let decoded = Hdlr::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_hdlr_empty() {
        let expected = Hdlr {
            handler_type: FourCC::new(b"vide"),
            name: String::new(),
        };
        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let decoded = Hdlr::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
