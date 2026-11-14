use crate::*;

/// SchemeInformationBox (`schi`), ISO/IEC 14496-12:2026 Section 13.4.7.
/// Its contents depend on the scheme; Common Encryption puts a `tenc` here.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Schi {
    pub tenc: Option<Tenc>,
    /// Every other child, kept verbatim and written back after `tenc`: other
    /// schemes put their own boxes here (OMA DRM's `odkm`, for example).
    pub unknown: Vec<Any>,
}

impl Eq for Schi {}

impl Atom for Schi {
    const KIND: FourCC = FourCC::new(b"schi");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let mut tenc = None;
        let mut unknown = Vec::new();
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Tenc(atom) => tenc.insert_once(atom)?,
                atom => unknown.push(atom),
            }
        }
        skip_trailing_padding(buf);

        Ok(Schi { tenc, unknown })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.tenc.encode(buf)?;
        self.unknown.encode(buf)
    }
}
