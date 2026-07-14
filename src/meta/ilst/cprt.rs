use crate::*;

/// An iTunes-style copyright notice, written by FFmpeg and iTunes for the
/// `copyright` metadata key.
///
/// This lives inside `moov/udta/meta/ilst` and shares its fourcc with the
/// unrelated ISO CopyrightBox ([`Cprt`]) found directly under `udta` — the
/// two have completely different layouts. It is deliberately NOT part of the
/// global [`Any`] dispatch; [`Ilst`] decodes it by header.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Copyright {
    pub country_indicator: u16,
    pub language_indicator: u16,
    pub text: String,
}

impl Atom for Copyright {
    const KIND: FourCC = FourCC::new(b"cprt");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let data = super::data::decode_text(buf)?;
        Ok(Copyright {
            country_indicator: data.country_indicator,
            language_indicator: data.language_indicator,
            text: data.text,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        super::data::encode_text(
            self.country_indicator,
            self.language_indicator,
            &self.text,
            buf,
        )
    }
}
