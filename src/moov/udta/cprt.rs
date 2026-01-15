use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cprt {
    pub language: String,
    pub notice: String,
}

impl AtomExt for Cprt {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"cprt");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let language_code = u16::decode(buf)?;
        let language = language_string(language_code);

        Ok(Cprt {
            language,
            notice: String::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let language_code = language_code(&self.language);
        language_code.encode(buf)?;
        self.notice.as_str().encode(buf)?;
        Ok(())
    }
}
