pub use crate::*;

ext! {
    name: Url,
    versions: [0],
    flags: {
        self_contained = 1,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Url {
    pub location: String,
}

impl AtomExt for Url {
    type Ext = UrlExt;

    const KIND_EXT: FourCC = FourCC::new(b"url ");

    fn decode_atom_ext(buf: &mut Bytes, _ext: UrlExt) -> Result<Self> {
        let location = match buf.is_empty() {
            true => "".to_string(),
            false => buf.decode()?,
        };

        Ok(Url { location })
    }

    fn encode_atom_ext(&self, buf: &mut BytesMut) -> Result<UrlExt> {
        if !self.location.is_empty() {
            self.location.as_str().encode(buf)?;
        }

        Ok(UrlExt {
            // TODO what does this do?
            self_contained: true,
            ..Default::default()
        })
    }
}
