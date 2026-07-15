use crate::*;

/// An iTunes-style `desc` (description) metadata item.
///
/// Like the other iTunes `ilst` text items ([`Copyright`], [`Tool`]), the value
/// is normally wrapped in a nested `data` atom (`desc → data → text`); FFmpeg
/// also emits a "short" style where the value follows the item header directly.
/// Both layouts are handled by the shared [`data`](super::data) codec. Decoding
/// the `data`-wrapped form as a bare string (as the previous implementation did)
/// consumed only up to the first NUL of the `data` box's own length field and
/// failed the whole `moov` with `UnderDecode(desc)`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Desc {
    pub country_indicator: u16,
    pub language_indicator: u16,
    pub text: String,
}

impl Atom for Desc {
    const KIND: FourCC = FourCC::new(b"desc");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let data = super::data::decode_text(buf)?;
        Ok(Desc {
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

    // Long QuickTime/GPAC/iTunes style: the value is wrapped in a nested `data`
    // atom. Decoding this as a raw string previously failed with
    // `UnderDecode(desc)` (the `data` box's length starts with a NUL).
    const ENCODED_DESC_LONG: &[u8] = &[
        0x00, 0x00, 0x00, 0x1E, // desc size = 30
        b'd', b'e', b's', b'c', //
        0x00, 0x00, 0x00, 0x16, // data size = 22
        b'd', b'a', b't', b'a', //
        0x00, 0x00, 0x00, 0x01, // type indicator: UTF-8
        0x00, 0x00, 0x00, 0x00, // country + language
        b'a', b' ', b'd', b'e', b's', b'c',
    ];

    #[test]
    fn test_desc_long_style() {
        let decoded = Desc::decode(&mut &ENCODED_DESC_LONG[..]).unwrap();
        assert_eq!(decoded.country_indicator, 0);
        assert_eq!(decoded.language_indicator, 0);
        assert_eq!(decoded.text, "a desc");

        // The encoder always emits the long `data`-wrapped layout.
        let mut out = Vec::new();
        decoded.encode(&mut out).unwrap();
        assert_eq!(out, ENCODED_DESC_LONG);
    }

    // Short FFmpeg style: the UTF-8 value follows the item header directly.
    #[test]
    fn test_desc_short_style() {
        let buf = [
            0x00, 0x00, 0x00, 0x16, // desc size = 22
            b'd', b'e', b's', b'c', //
            0x00, 0x00, 0x00, 0x01, // type indicator (short style): UTF-8
            0x00, 0x00, 0x00, 0x00, // country + language
            b'a', b' ', b'd', b'e', b's', b'c',
        ];
        let decoded = Desc::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded.country_indicator, 0);
        assert_eq!(decoded.language_indicator, 0);
        assert_eq!(decoded.text, "a desc");
    }
}
