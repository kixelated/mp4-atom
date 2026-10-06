use crate::*;

ext! {
    name: Url,
    versions: [0],
    flags: {
        self_contained = 0,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Url {
    pub location: String,
}

impl AtomExt for Url {
    type Ext = UrlExt;

    const KIND_EXT: FourCC = FourCC::new(b"url ");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: UrlExt) -> Result<Self> {
        let location = match buf.has_remaining() {
            true => String::decode(buf)?,
            false => "".to_string(),
        };

        Ok(Url { location })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<UrlExt> {
        if !self.location.is_empty() {
            self.location.as_str().encode(buf)?;
        }

        Ok(UrlExt {
            // ISOBMFF §8.7.2: flag bit 0 = media data is in the same file
            self_contained: self.location.is_empty(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENCODED_EMPTY: &[u8] = &[
        0x00, 0x00, 0x00, 0x0c, b'u', b'r', b'l', b' ', 0x00, 0x00, 0x00, 0x01,
    ];
    #[test]
    fn test_url_empty_encode() {
        let url = Url {
            location: "".into(),
        };

        let mut buf = Vec::new();
        url.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_EMPTY);
    }

    #[test]
    fn test_url_empty_decode() {
        let buf = &mut std::io::Cursor::new(&ENCODED_EMPTY);

        let url = Url::decode(buf).expect("failed to decode url");

        assert_eq!(
            url,
            Url {
                location: "".into(),
            }
        );
    }

    const ENCODED_HTTP: &[u8] = &[
        0x00, 0x00, 0x00, 0x2a, b'u', b'r', b'l', b' ', 0x00, 0x00, 0x00, 0x00, b'h', b't', b't',
        b'p', b's', b':', b'/', b'/', b'e', b'x', b'a', b'm', b'p', b'l', b'e', b'.', b'c', b'o',
        b'm', b'/', b'd', b'a', b't', b'a', b'.', b'b', b'l', b'o', b'b', 0,
    ];
    #[test]
    fn test_url_http_encode() {
        let url = Url {
            location: "https://example.com/data.blob".into(),
        };

        let mut buf = Vec::new();
        url.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_HTTP);
    }

    #[test]
    fn test_url_http_decode() {
        let buf = &mut std::io::Cursor::new(&ENCODED_HTTP);

        let url = Url::decode(buf).expect("failed to decode url");
        assert_eq!(
            url,
            Url {
                location: "https://example.com/data.blob".into(),
            }
        );
    }
}
