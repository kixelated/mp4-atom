use crate::*;

pub(crate) const DATA_4CC: FourCC = FourCC::new(b"data");
const TYPE_INDICATOR_UTF8: u32 = 1u32;

/// A UTF-8 text payload of an iTunes-style `ilst` metadata item.
///
/// Two encodings exist in the wild: the FFmpeg "short" style, where the value
/// follows the item header directly, and the QuickTime/GPAC "long" style,
/// where the value is wrapped in a nested `data` atom.
pub(crate) struct DataText {
    pub country_indicator: u16,
    pub language_indicator: u16,
    pub text: String,
}

pub(crate) fn decode_text<B: Buf>(buf: &mut B) -> Result<DataText> {
    let type_indicator_or_len = u32::decode(buf)?;
    match type_indicator_or_len {
        1 => {
            // Too short for a valid length, so probably
            // UTF-8 text, FFmpeg short-style
            let country_indicator = u16::decode(buf)?;
            let language_indicator = u16::decode(buf)?;
            Ok(DataText {
                country_indicator,
                language_indicator,
                text: String::decode(buf)?,
            })
        }
        _ => {
            // Maybe Atom follows on straight away.
            // Try parsing as Quicktime data atom: GPAC style or FFmpeg long style
            let fourcc = FourCC::decode(buf)?;
            if fourcc != DATA_4CC {
                return Err(Error::UnexpectedBox(fourcc));
            }
            let type_indicator = u32::decode(buf)?;
            if type_indicator != TYPE_INDICATOR_UTF8 {
                return Err(Error::Unsupported(
                    "Only UTF-8 text is supported in ilst data atoms",
                ));
            }
            let country_indicator = u16::decode(buf)?;
            let language_indicator = u16::decode(buf)?;
            let remaining_bytes = buf.remaining();
            let body = &mut buf.slice(remaining_bytes);
            let text = String::from_utf8(body.to_vec()).map_err(|_| Error::InvalidSize)?;
            buf.advance(remaining_bytes);
            Ok(DataText {
                country_indicator,
                language_indicator,
                text,
            })
        }
    }
}

pub(crate) fn encode_text<B: BufMut>(
    country_indicator: u16,
    language_indicator: u16,
    text: &str,
    buf: &mut B,
) -> Result<()> {
    let text_bytes = text.as_bytes();
    // the length of the nested atom is the length field (4 bytes),
    // the 4CC (4 bytes), the type indicator (4 bytes), the country
    // indicator (2 bytes), the language indicator (2 bytes) and
    // then the actual text.
    let nested_len = (4 + 4 + 4 + 2 + 2 + text_bytes.len()) as u32;
    nested_len.encode(buf)?;
    DATA_4CC.encode(buf)?;
    TYPE_INDICATOR_UTF8.encode(buf)?;
    country_indicator.encode(buf)?;
    language_indicator.encode(buf)?;
    text_bytes.encode(buf)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Long QuickTime/GPAC style: the value is wrapped in a nested `data` atom.
    // This is the layout `encode_text` always emits.
    const LONG_STYLE: &[u8] = &[
        0x00, 0x00, 0x00, 0x1A, // data atom size = 26
        b'd', b'a', b't', b'a', //
        0x00, 0x00, 0x00, 0x01, // type indicator: UTF-8
        0x00, 0x00, 0x00, 0x00, // country + language
        b'(', b'c', b')', b' ', b'2', b'0', b'2', b'6', b' ', b'x',
    ];

    #[test]
    fn test_decode_text_short_style() {
        // FFmpeg short style: the leading `1` is the type indicator (too small
        // to be a valid `data` atom length), so the UTF-8 value follows directly.
        let buf = [
            0x00, 0x00, 0x00, 0x01, // type indicator (short style): UTF-8
            0x00, 0x00, 0x00, 0x00, // country + language
            b'2', b'0', b'2', b'6',
        ];
        let decoded = decode_text(&mut &buf[..]).unwrap();
        assert_eq!(decoded.country_indicator, 0);
        assert_eq!(decoded.language_indicator, 0);
        assert_eq!(decoded.text, "2026");
    }

    #[test]
    fn test_decode_text_long_style() {
        let decoded = decode_text(&mut &LONG_STYLE[..]).unwrap();
        assert_eq!(decoded.country_indicator, 0);
        assert_eq!(decoded.language_indicator, 0);
        assert_eq!(decoded.text, "(c) 2026 x");
    }

    #[test]
    fn test_encode_text_emits_long_style() {
        // `encode_text` always emits the long `data`-wrapped layout, which
        // decodes back to the same value.
        let mut buf = Vec::new();
        encode_text(0, 0, "(c) 2026 x", &mut buf).unwrap();
        assert_eq!(buf, LONG_STYLE);

        let decoded = decode_text(&mut buf.as_slice()).unwrap();
        assert_eq!(decoded.text, "(c) 2026 x");
    }
}
