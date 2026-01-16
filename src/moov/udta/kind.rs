use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Kind {
    pub scheme_uri: String,
    pub value: String,
}

impl AtomExt for Kind {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"kind");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        Ok(Kind {
            scheme_uri: String::decode(buf)?,
            value: String::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.scheme_uri.as_str().encode(buf)?;
        self.value.as_str().encode(buf)?;
        Ok(())
    }
}
