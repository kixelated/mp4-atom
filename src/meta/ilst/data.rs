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
