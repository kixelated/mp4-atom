mod url;
mod urn;
pub use url::*;
pub use urn::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dref {
    pub entries: Vec<DrefEntry>,
}

/// An entry in a data reference box.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum DrefEntry {
    Url(Url),
    Urn(Urn),
    Unknown(FourCC, Vec<u8>),
}

impl DrefEntry {
    pub fn kind(&self) -> FourCC {
        match self {
            Self::Url(_) => Url::KIND,
            Self::Urn(_) => Urn::KIND,
            Self::Unknown(kind, _) => *kind,
        }
    }
}

impl Decode for DrefEntry {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        let header = Header::decode(buf)?;
        let size = header.size.unwrap_or(buf.remaining());
        if size > buf.remaining() {
            return Err(Error::OutOfBounds);
        }

        match header.kind {
            kind if kind == Url::KIND => Url::decode_atom(&header, buf).map(Self::Url),
            kind if kind == Urn::KIND => Urn::decode_atom(&header, buf).map(Self::Urn),
            kind => {
                let data = Vec::decode(&mut buf.slice(size))?;
                buf.advance(size);
                Ok(Self::Unknown(kind, data))
            }
        }
    }
}

impl Encode for DrefEntry {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        match self {
            Self::Url(url) => url.encode(buf),
            Self::Urn(urn) => urn.encode(buf),
            Self::Unknown(kind, data) => {
                Header {
                    kind: *kind,
                    size: Some(data.len()),
                }
                .encode(buf)?;
                data.encode(buf)
            }
        }
    }
}

impl From<Url> for DrefEntry {
    fn from(url: Url) -> Self {
        Self::Url(url)
    }
}

impl From<Urn> for DrefEntry {
    fn from(urn: Urn) -> Self {
        Self::Urn(urn)
    }
}

impl AtomExt for Dref {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"dref");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let entry_count = u32::decode(buf)?;
        let mut entries = Vec::new();

        for _ in 0..entry_count {
            entries.push(DrefEntry::decode(buf)?);
        }

        Ok(Dref { entries })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let entry_count =
            u32::try_from(self.entries.len()).map_err(|_| Error::TooLarge(Self::KIND))?;
        entry_count.encode(buf)?;

        for entry in &self.entries {
            entry.encode(buf)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const ENCODED_URN_AND_UNKNOWN: &[u8] = &[
        0, 0, 0, 48, b'd', b'r', b'e', b'f', 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 21, b'u', b'r', b'n',
        b' ', 0, 0, 0, 0, b'n', b'a', b'm', b'e', 0, b'l', b'o', b'c', 0, 0, 0, 0, 11, b'a', b'b',
        b'c', b'd', 1, 2, 3,
    ];

    #[test]
    fn decode_and_preserve_urn_and_unknown_entries() {
        let dref = Dref::decode(&mut Cursor::new(ENCODED_URN_AND_UNKNOWN)).unwrap();

        assert_eq!(
            dref,
            Dref {
                entries: vec![
                    Urn {
                        name: "name".into(),
                        location: "loc".into(),
                    }
                    .into(),
                    DrefEntry::Unknown(FourCC::new(b"abcd"), vec![1, 2, 3]),
                ],
            }
        );

        let mut encoded = Vec::new();
        dref.encode(&mut encoded).unwrap();
        assert_eq!(encoded, ENCODED_URN_AND_UNKNOWN);
    }
}
