use crate::*;

ext! {
    name: Urn,
    versions: [0],
    flags: {}
}

/// A name-based data reference and its location.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Urn {
    pub name: String,
    pub location: String,
}

impl AtomExt for Urn {
    type Ext = UrnExt;

    const KIND_EXT: FourCC = FourCC::new(b"urn ");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: UrnExt) -> Result<Self> {
        Ok(Self {
            name: String::decode(buf)?,
            location: String::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<UrnExt> {
        self.name.as_str().encode(buf)?;
        self.location.as_str().encode(buf)?;
        Ok(UrnExt::default())
    }
}
