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

#[cfg(test)]
mod tests {
    use super::*;

    // Long QuickTime/GPAC style: the value is wrapped in a nested `data` atom.
    // This is what FFmpeg and iTunes write for the `copyright` key.
    const ENCODED_CPRT_LONG: &[u8] = &[
        0x00, 0x00, 0x00, 0x22, // cprt size = 34
        b'c', b'p', b'r', b't', //
        0x00, 0x00, 0x00, 0x1A, // data size = 26
        b'd', b'a', b't', b'a', //
        0x00, 0x00, 0x00, 0x01, // type indicator: UTF-8
        0x00, 0x00, 0x00, 0x00, // country + language
        b'(', b'c', b')', b' ', b'2', b'0', b'2', b'6', b' ', b'x',
    ];

    #[test]
    fn test_copyright_long_style() {
        let decoded = Copyright::decode(&mut &ENCODED_CPRT_LONG[..]).unwrap();
        assert_eq!(decoded.country_indicator, 0);
        assert_eq!(decoded.language_indicator, 0);
        assert_eq!(decoded.text, "(c) 2026 x");

        // The encoder always emits the long `data`-wrapped layout.
        let mut out = Vec::new();
        decoded.encode(&mut out).unwrap();
        assert_eq!(out, ENCODED_CPRT_LONG);
    }

    // Short FFmpeg style: the UTF-8 value follows the item header directly (no
    // nested `data` atom); the leading `1` is the type indicator, too small to
    // be a valid `data` atom length.
    #[test]
    fn test_copyright_short_style() {
        let buf = [
            0x00, 0x00, 0x00, 0x14, // cprt size = 20
            b'c', b'p', b'r', b't', //
            0x00, 0x00, 0x00, 0x01, // type indicator (short style): UTF-8
            0x00, 0x00, 0x00, 0x00, // country + language
            b'2', b'0', b'2', b'6',
        ];
        let decoded = Copyright::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded.country_indicator, 0);
        assert_eq!(decoded.language_indicator, 0);
        assert_eq!(decoded.text, "2026");
    }
}
