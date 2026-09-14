use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct C2pa {
    pub box_purpose: String,
    pub data: Vec<u8>,
}

impl UuidAtomExt for C2pa {
    const EXTENDED_TYPE_EXT: ExtendedType = ExtendedType::new(&[
        0xD8, 0xFE, 0xC3, 0xD6, 0x1B, 0x0E, 0x48, 0x3C, 0x92, 0x97, 0x58, 0x28, 0x87, 0x7E, 0xC4,
        0x81,
    ]);

    // The ext version is restricted to 0, and flags 0 for C2PA, so they are ignored
    type Ext = ();

    fn decode_uuid_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        Ok(C2pa {
            box_purpose: String::decode(buf)?,
            data: Vec::decode(buf)?,
        })
    }

    fn encode_uuid_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.box_purpose.as_str().encode(buf)?;
        self.data.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Testing the encoding and decoding logic
    #[test]
    fn test_round_trip() -> Result<()> {
        let input = C2pa {
            box_purpose: String::from("uuid:test"),
            data: vec![0xDE, 0xAD, 0xBE, 0xEF],
        };

        let mut buf = Vec::new();

        let _ = C2pa::encode_uuid_body_ext(&input, &mut buf)?;

        let mut cursor = &buf[..];

        let ouput = C2pa::decode_uuid_body_ext(&mut cursor, ())?;

        assert_eq!(input, ouput);

        Ok(())
    }
}
