use crate::*;

const DATA_4CC: FourCC = FourCC::new(b"data");
const TYPE_INDICATOR_UTF8: u32 = 1u32;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tool {
    pub country_indicator: u16,
    pub language_indicator: u16,
    pub text: String,
}

impl Tool {
    pub fn new(country_indicator: u16, language_indicator: u16, text: String) -> Result<Self> {
        Ok(Self {
            country_indicator,
            language_indicator,
            text,
        })
    }
}

impl Atom for Tool {
    const KIND: FourCC = FourCC::new(b"\xa9too");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let type_indicator_or_len = u32::decode(buf)?;
        match type_indicator_or_len {
            1 => {
                // Too short for a valid length, so probably
                // UTF-8 text, FFmpeg short-style
                let country_indicator = u16::decode(buf)?;
                let language_indicator = u16::decode(buf)?;
                Ok(Tool {
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
                        "Only UTF-8 text is support in ilst tool box",
                    ));
                }
                let country_indicator = u16::decode(buf)?;
                let language_indicator = u16::decode(buf)?;
                let remaining_bytes = (type_indicator_or_len - 16) as usize;
                let body = &mut buf.slice(remaining_bytes);
                let text = String::from_utf8(body.to_vec()).unwrap();
                buf.advance(remaining_bytes);
                Ok(Tool {
                    country_indicator,
                    language_indicator,
                    text,
                })
            }
        }
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let text_bytes = self.text.as_bytes();
        let nested_len = (4 + 4 + 4 + 2 + 2 + text_bytes.len()) as u32;
        nested_len.encode(buf)?;
        DATA_4CC.encode(buf)?;
        TYPE_INDICATOR_UTF8.encode(buf)?;
        self.country_indicator.encode(buf)?;
        self.language_indicator.encode(buf)?;
        text_bytes.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ilst_tool_short_style() {
        let buf = vec![
            0, 0, 0, 28, 0xa9, b't', b'o', b'o', 0, 0, 0, 1, 0, 0, 0, 0, b'L', b'a', b'v', b'f',
            b'6', b'1', b'.', b'7', b'.', b'1', b'0', b'0',
        ];

        let parse_result = Tool::decode(&mut buf.as_slice());
        assert!(parse_result.is_ok());
        let ctoo = parse_result.unwrap();
        assert_eq!(ctoo.country_indicator, 0);
        assert_eq!(ctoo.language_indicator, 0);
        assert_eq!(ctoo.text, "Lavf61.7.100");
    }

    const ENCODED_CTOO: &[u8] = &[
        0x00, 0x00, 0x00, 0x24, 0xA9, 0x74, 0x6F, 0x6F, 0x00, 0x00, 0x00, 0x1C, b'd', b'a', b't',
        b'a', 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, b'L', b'a', b'v', b'f', b'6', b'1',
        b'.', b'7', b'.', b'1', b'0', b'0',
    ];
    #[test]
    fn test_ilst_tool_long_style() {
        let inbuf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_CTOO);
        let parse_result = Tool::decode(inbuf);
        assert!(parse_result.is_ok());
        let ctoo = parse_result.unwrap();
        assert_eq!(ctoo.country_indicator, 0);
        assert_eq!(ctoo.language_indicator, 0);
        assert_eq!(ctoo.text, "Lavf61.7.100");

        let mut outbuf = Vec::new();
        ctoo.encode(&mut outbuf).unwrap();

        assert_eq!(outbuf.as_slice(), ENCODED_CTOO);
    }
}
