use crate::*;

ext! {
    name: Ainf,
    versions: [0],
    flags: {
        hidden = 0,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ainf {
    pub hidden: bool,
    // TODO: this could be parsed further if DECE ever becomes important again
    pub profile_version: u32,
    pub apid: String,
}

impl AtomExt for Ainf {
    const KIND_EXT: FourCC = FourCC::new(b"ainf");

    type Ext = AinfExt;

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: AinfExt) -> Result<Self> {
        let profile_version = u32::decode(buf)?;
        let apid = String::decode(buf)?;
        let remaining = buf.remaining();
        if remaining != 0 {
            tracing::warn!("Found additional data in ainf box, which could be additional private boxes, and that data is being ignored.");
            buf.advance(remaining);
        }
        Ok(Ainf {
            hidden: ext.hidden,
            profile_version,
            apid,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<AinfExt> {
        let ext = AinfExt {
            hidden: self.hidden,
            version: AinfVersion::V0,
        };
        self.profile_version.encode(buf)?;
        self.apid.as_str().encode(buf)?;
        Ok(ext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENCODED_AINF: &[u8] = &[
        0x00, 0x00, 0x00, 0x38, 0x61, 0x69, 0x6e, 0x66, 0x00, 0x00, 0x00, 0x00, 0x68, 0x31, 0x30,
        0x37, 0x75, 0x72, 0x6e, 0x3a, 0x64, 0x65, 0x63, 0x65, 0x3a, 0x61, 0x70, 0x69, 0x64, 0x3a,
        0x6f, 0x72, 0x67, 0x3a, 0x64, 0x65, 0x63, 0x65, 0x63, 0x76, 0x70, 0x3a, 0x64, 0x65, 0x76,
        0x69, 0x63, 0x65, 0x30, 0x30, 0x32, 0x2d, 0x34, 0x76, 0x37, 0x00,
    ];

    #[test]
    fn test_ainf_decode() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_AINF);
        let ainf = Ainf::decode(buf).expect("failed to decode ainf");
        assert_eq!(
            ainf,
            Ainf {
                hidden: false,
                profile_version: 1748054071,
                apid: "urn:dece:apid:org:dececvp:device002-4v7".into(),
            }
        );
    }

    #[test]
    fn test_ainf_encode() {
        let ainf = Ainf {
            hidden: false,
            profile_version: 1748054071,
            apid: "urn:dece:apid:org:dececvp:device002-4v7".into(),
        };

        let mut buf = Vec::new();
        ainf.encode(&mut buf).unwrap();
        assert_eq!(buf, ENCODED_AINF);
    }
}
