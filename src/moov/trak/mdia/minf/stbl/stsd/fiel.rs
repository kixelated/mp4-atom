use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Fiel {
    pub field_count: u8,
    pub field_order: u8,
}

impl Fiel {
    pub fn new(field_count: u8, field_order: u8) -> Result<Self> {
        Ok(Fiel {
            field_count,
            field_order,
        })
    }
}

impl Atom for Fiel {
    const KIND: FourCC = FourCC::new(b"fiel");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let field_count = u8::decode(buf)?;
        let field_order = u8::decode(buf)?;

        Ok(Fiel {
            field_count,
            field_order,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.field_count.encode(buf)?;
        self.field_order.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENCODED_FIEL: &[u8] = &[0x00, 0x00, 0x00, 0x0a, b'f', b'i', b'e', b'l', 0x01, 0x06];

    #[test]
    fn test_fiel_decode() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_FIEL);

        let fiel = Fiel::decode(buf).expect("failed to decode fiel");

        assert_eq!(
            fiel,
            Fiel {
                field_count: 1,
                field_order: 6,
            }
        );
    }

    #[test]
    fn test_fiel_encode() {
        let fiel = Fiel {
            field_count: 1,
            field_order: 6,
        };

        let mut buf = Vec::new();
        fiel.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_FIEL);
    }
}
