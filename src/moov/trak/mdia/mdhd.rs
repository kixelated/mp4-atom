use crate::*;

ext! {
    name: Mdhd,
    versions: [0, 1],
    flags: {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mdhd {
    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,
    pub language: String,
}

impl AtomExt for Mdhd {
    const KIND: FourCC = FourCC::new(b"mdhd");
    type Ext = MdhdExt;

    fn decode_atom(buf: &mut Buf, ext: MdhdExt) -> Result<Self> {
        let (creation_time, modification_time, timescale, duration) = match ext.version {
            MdhdVersion::V1 => (
                u64::decode(buf)?,
                u64::decode(buf)?,
                u32::decode(buf)?,
                u64::decode(buf)?,
            ),
            MdhdVersion::V0 => (
                u32::decode(buf)? as u64,
                u32::decode(buf)? as u64,
                u32::decode(buf)?,
                u32::decode(buf)? as u64,
            ),
        };

        let language_code = u16::decode(buf)?;
        let language = language_string(language_code);

        Ok(Mdhd {
            creation_time,
            modification_time,
            timescale,
            duration,
            language,
        })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<MdhdExt> {
        self.creation_time.encode(buf)?;
        self.modification_time.encode(buf)?;
        self.timescale.encode(buf)?;
        self.duration.encode(buf)?;

        let language_code = language_code(&self.language);
        buf.u16(language_code)?;
        buf.u16(0)?; // pre-defined

        Ok(MdhdVersion::V1.into())
    }
}

fn language_string(language: u16) -> String {
    let mut lang: [u16; 3] = [0; 3];

    lang[0] = ((language >> 10) & 0x1F) + 0x60;
    lang[1] = ((language >> 5) & 0x1F) + 0x60;
    lang[2] = ((language) & 0x1F) + 0x60;

    // Decode utf-16 encoded bytes into a string.
    let lang_str = decode_utf16(lang.iter().cloned())
        .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
        .collect::<String>();

    lang_str
}

fn language_code(language: &str) -> u16 {
    let mut lang = language.encode_utf16();
    let mut code = (lang.next().unwrap_or(0) & 0x1F) << 10;
    code += (lang.next().unwrap_or(0) & 0x1F) << 5;
    code += lang.next().unwrap_or(0) & 0x1F;
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_language_code(lang: &str) {
        let code = language_code(lang);
        let lang2 = language_string(code);
        assert_eq!(lang, lang2);
    }

    #[test]
    fn test_language_codes() {
        test_language_code("und");
        test_language_code("eng");
        test_language_code("kor");
    }

    #[test]
    fn test_mdhd32() {
        let expected = Mdhd {
            creation_time: 100,
            modification_time: 200,
            timescale: 48000,
            duration: 30439936,
            language: String::from("und"),
        };
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Mdhd::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_mdhd64() {
        let expected = Mdhd {
            creation_time: 100,
            modification_time: 200,
            timescale: 48000,
            duration: 30439936,
            language: String::from("eng"),
        };
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Mdhd::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
