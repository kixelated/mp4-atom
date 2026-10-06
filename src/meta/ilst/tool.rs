use crate::*;

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
        let data = super::data::decode_text(buf)?;
        Ok(Tool {
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

    #[test]
    fn test_bad_nested_atom() {
        let buf = vec![
            0x00, 0x00, 0x00, 28, 0xA9, 0x74, 0x6F, 0x6F, 0x00, 0x00, 0x00, 0x1C, b'd', b'a', b't',
            b'x', 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, b't', b'e', b's', b't',
        ];

        let parse_result = Tool::decode(&mut buf.as_slice());
        assert!(parse_result.is_err());
        match parse_result.err().unwrap() {
            Error::UnexpectedBox(four_cc) => {
                assert_eq!(four_cc, FourCC::new(b"datx"));
            }
            _ => {
                panic!("unexpected error");
            }
        }
    }

    #[test]
    fn test_bad_type_indicator() {
        let buf = vec![
            0x00, 0x00, 0x00, 28, 0xA9, 0x74, 0x6F, 0x6F, 0x00, 0x00, 0x00, 0x1C, b'd', b'a', b't',
            b'a', 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, b't', b'e', b's', b't',
        ];

        let parse_result = Tool::decode(&mut buf.as_slice());
        assert!(parse_result.is_err());
        match parse_result.err().unwrap() {
            Error::Unsupported(s) => {
                assert_eq!(s, "Only UTF-8 text is supported in ilst data atoms")
            }
            _ => {
                panic!("unexpected error");
            }
        }
    }
}
