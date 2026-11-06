use crate::*;

ext! {
    name: Schm,
    versions: [0],
    flags: {
        scheme_uri_present = 0,
    }
}

/// SchemeTypeBox (`schm`), ISO/IEC 14496-12:2026 Sect 13.4.6: the protection or
/// restriction scheme, such as `cenc` or `cbcs` for Common Encryption (ISO/IEC 23001-7).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Schm {
    pub scheme_type: FourCC,
    /// The major version in the upper 16 bits, the minor in the lower; `0x00010000` for
    /// Common Encryption.
    pub scheme_version: u32,
    pub scheme_uri: Option<String>,
}

impl AtomExt for Schm {
    type Ext = SchmExt;

    const KIND_EXT: FourCC = FourCC::new(b"schm");

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: SchmExt) -> Result<Self> {
        let scheme_type = FourCC::decode(buf)?;
        let scheme_version = u32::decode(buf)?;
        let scheme_uri = match ext.scheme_uri_present {
            true => Some(String::decode(buf)?),
            false => None,
        };

        Ok(Schm {
            scheme_type,
            scheme_version,
            scheme_uri,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<SchmExt> {
        self.scheme_type.encode(buf)?;
        self.scheme_version.encode(buf)?;
        if let Some(uri) = &self.scheme_uri {
            if uri.contains('\0') {
                return Err(Error::InvalidString("schm URI contains a NUL".into()));
            }
            uri.as_str().encode(buf)?;
        }

        Ok(SchmExt {
            version: SchmVersion::V0,
            scheme_uri_present: self.scheme_uri.is_some(),
        })
    }
}
