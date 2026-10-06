use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rtng {
    pub entity: FourCC,
    pub criteria: FourCC,
    pub language: String,
    pub rating_info: String,
}

impl AtomExt for Rtng {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"rtng");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let entity = FourCC::decode(buf)?;
        let criteria = FourCC::decode(buf)?;
        let language_code = u16::decode(buf)?;
        let language = language_string(language_code);
        let num_remaining_bytes = buf.remaining();
        let remaining_bytes = &mut buf.slice(num_remaining_bytes);
        let mut rating_info =
            String::from_utf8(remaining_bytes.to_vec()).map_err(|_| Error::InvalidSize)?;
        if rating_info.ends_with('\0') {
            rating_info.truncate(rating_info.len() - 1);
        }
        buf.advance(num_remaining_bytes);
        Ok(Rtng {
            entity,
            criteria,
            language,
            rating_info,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.entity.encode(buf)?;
        self.criteria.encode(buf)?;
        let language_code = language_code(&self.language);
        language_code.encode(buf)?;
        self.rating_info.as_str().encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    const ENCODED_RTNG: &[u8] = &[
        0x00, 0x00, 0x00, 0x1d, 0x72, 0x74, 0x6e, 0x67, 0x00, 0x00, 0x00, 0x00, 0x4d, 0x50, 0x41,
        0x41, 0x00, 0x00, 0x00, 0x00, 0x15, 0xc7, 0x47, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn test_rtng_decode() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_RTNG);
        let rtng = Rtng::decode(buf).expect("failed to decode rtng");
        assert_eq!(
            rtng,
            Rtng {
                entity: b"MPAA".into(),
                criteria: b"\0\0\0\0".into(),
                language: "eng".into(),
                rating_info: "G\0\0\0\0\0".into(),
            }
        );
    }

    #[test]
    fn test_rtng_encode() {
        let rtng = Rtng {
            entity: b"MPAA".into(),
            criteria: b"\0\0\0\0".into(),
            language: "eng".into(),
            rating_info: "G\0\0\0\0\0".into(),
        };

        let mut buf = Vec::new();
        rtng.encode(&mut buf).unwrap();
        assert_eq!(buf, ENCODED_RTNG);
    }
}
