use crate::*;

ext! {
    name: C2pa,
    versions: [0],
    flags: {}
}

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

    // implement type, and get an understanding of what it does TODO
    type Ext = C2paExt;

    fn decode_uuid_body_ext<B: Buf>(buf: &mut B, _: C2paExt) -> Result<Self> {
        // The ext version is restricted to 0, and flags 0 for C2PA, so they are ignored
        Ok(C2pa {
            box_purpose: String::decode(buf)?,
            data: Vec::decode(buf)?,
        })
    }

    fn encode_uuid_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<C2paExt> {
        self.box_purpose.as_str().encode(buf)?;
        self.data.encode(buf)?;
        Ok(C2paVersion::V0.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip() -> Result<()> {
        Ok(())
    }
}
