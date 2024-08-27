mod url;
pub use url::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Dref {
    pub urls: Vec<Url>,
}

impl AtomExt for Dref {
    type Ext = ();

    const KIND: FourCC = FourCC::new(b"dref");

    fn decode_atom(buf: &mut Buf, _ext: ()) -> Result<Self> {
        let entry_count = u32::decode(buf)?;
        let mut urls = Vec::new();

        for _ in 0..entry_count {
            let url = Url::decode(buf)?;
            urls.push(url);
        }

        Ok(Dref { urls })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        (self.urls.len() as u32).encode(buf)?;

        for url in &self.urls {
            url.encode(buf)?;
        }

        Ok(())
    }
}
