use crate::*;

pub trait UuidAtom: Sized {
    const EXTENDED_TYPE: ExtendedType;

    fn decode_uuid_body<B: Buf>(buf: &mut B) -> Result<Self>;
    fn encode_uuid_body<B: BufMut>(&self, buf: &mut B) -> Result<()>;
}

pub(crate) trait UuidAtomExt: Sized {
    const EXTENDED_TYPE_EXT: ExtendedType;
    type Ext: Ext;

    fn decode_uuid_body_ext<B: Buf>(buf: &mut B, ext: Self::Ext) -> Result<Self>;
    fn encode_uuid_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<Self::Ext>;
}

impl<T: UuidAtomExt> UuidAtom for T {
    const EXTENDED_TYPE: ExtendedType = Self::EXTENDED_TYPE_EXT;

    // logic borrowed from ./atom_ext.rs
    fn decode_uuid_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let ext = Ext::decode(u32::decode(buf)?)?;
        UuidAtomExt::decode_uuid_body_ext(buf, ext)
    }

    fn encode_uuid_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        // Here's the magic, we reserve space for the version/flags first
        let start = buf.len();
        0u32.encode(buf)?;

        // That way we can return them as part of the trait, avoiding boilerplate
        let ext = self.encode_uuid_body_ext(buf)?;

        // Go back and update the version/flags
        let header = ext.encode()?;
        buf.set_slice(start, &header.to_be_bytes());

        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Uuid {
    C2pa(C2pa),
    Unknown(ExtendedType, Vec<u8>),
}

// This should be made in a macro to support all defined box types TODO
impl Atom for Uuid {
    const KIND: FourCC = FourCC::new(b"uuid");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let et = ExtendedType::decode(buf)?;

        match et {
            C2pa::EXTENDED_TYPE => Ok(Uuid::C2pa(C2pa::decode_uuid_body(buf)?)),
            _ => {
                // Consider changing to use bytes lib
                // implementation for efficency TODO
                let payload = Vec::<u8>::decode(buf)?;
                Ok(Uuid::Unknown(et, payload))
            }
        }
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        match self {
            Uuid::C2pa(c2pa) => {
                C2pa::EXTENDED_TYPE.encode(buf)?;
                c2pa.encode_uuid_body(buf)?;
            }
            Uuid::Unknown(et, payload) => {
                et.encode(buf)?;
                payload.encode(buf)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c2pa_round_trip() -> Result<()> {
        let original = Uuid::C2pa(C2pa {
            box_purpose: "urn:uuid:...".to_string(),
            data: vec![1, 2, 3, 4],
        });

        // 1. Encode into a byte buffer
        let mut buf = Vec::new();
        original.encode_body(&mut buf)?;

        // 2. Decode back from the buffer
        let mut cursor = &buf[..];
        let decoded = Uuid::decode_body(&mut cursor)?;

        // 3. Assert equality
        assert_eq!(original, decoded, "They werent equal");
        Ok(())
    }

    #[test]
    fn test_c2pa_uuid_golden_vector() -> Result<()> {
        let mut bytes = Vec::new();

        // 1. C2pa Extended Type bytes (16 bytes from the specification)
        bytes.extend_from_slice(&[
            0xD8, 0xFE, 0xC3, 0xD6, 0x1B, 0x0E, 0x48, 0x3C, 0x92, 0x97, 0x58, 0x28, 0x87, 0x7E,
            0xC4, 0x81,
        ]);

        // 2. FullBox version and flags (version = 0, flags = 0 -> 4 bytes of 0x00)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        // 3. box_purpose string bytes (null-terminated UTF-8 string)
        bytes.extend_from_slice(b"urn:uuid:test\0");

        // 4. Data payload bytes (0xDEADBEEF)
        bytes.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);

        let mut cursor = &bytes[..];
        let decoded = Uuid::decode_body(&mut cursor)?;

        let expected = Uuid::C2pa(C2pa {
            box_purpose: "urn:uuid:test".to_string(),
            data: vec![0xDE, 0xAD, 0xBE, 0xEF],
        });

        assert_eq!(decoded, expected);
        Ok(())
    }
}
