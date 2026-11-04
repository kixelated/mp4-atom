mod frma;
mod schi;
mod schm;
mod sinf;
mod tenc;

pub use frma::*;
pub use schi::*;
pub use schm::*;
pub use sinf::*;
pub use tenc::*;

use crate::*;

/// A protected video sample entry (`encv`), ISO/IEC 14496-12:2022 Sect 8.12.
///
/// On the wire it is the original sample entry renamed to `encv`, with a `sinf`
/// added that names the original type in `frma`. Here it is that original entry,
/// decoded as usual, plus the `sinf`.
///
/// An original type this crate does not decode stays [`Codec::Unknown`], which has
/// no visual fields and cannot be encoded. Protected text, system and metadata
/// entries (`enct`, `encs`, `encm`) are not handled.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Encv {
    /// The original sample entry, of the type `sinf.frma` names.
    pub codec: Box<Codec>,
    pub sinf: Sinf,
}

/// A protected audio sample entry (`enca`), ISO/IEC 14496-12:2022 Sect 8.12.
///
/// See [`Encv`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Enca {
    /// The original sample entry, of the type `sinf.frma` names.
    pub codec: Box<Codec>,
    pub sinf: Sinf,
}

impl Atom for Encv {
    const KIND: FourCC = FourCC::new(b"encv");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let (codec, sinf) =
            decode_protected(buf, Self::KIND, |fields| Visual::decode(fields).map(|_| ()))?;
        check_kind(&codec, Self::KIND)?;
        Ok(Encv { codec, sinf })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        check_kind(&self.codec, Self::KIND)?;
        encode_protected(&self.codec, &self.sinf, buf)
    }
}

impl Atom for Enca {
    const KIND: FourCC = FourCC::new(b"enca");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let (codec, sinf) =
            decode_protected(buf, Self::KIND, |fields| Audio::decode(fields).map(|_| ()))?;
        check_kind(&codec, Self::KIND)?;
        Ok(Enca { codec, sinf })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        check_kind(&self.codec, Self::KIND)?;
        encode_protected(&self.codec, &self.sinf, buf)
    }
}

/// `encv` wraps a visual entry and `enca` an audio one; neither wraps a protected entry.
/// An entry this crate does not decode passes, since its kind is unknown.
fn check_kind(codec: &Codec, kind: FourCC) -> Result<()> {
    let ok = match codec {
        Codec::Unknown(..) => true,
        Codec::Encv(_) | Codec::Enca(_) => false,
        codec if kind == Encv::KIND => codec.visual().is_some(),
        codec => codec.audio().is_some(),
    };
    match ok {
        true => Ok(()),
        false if kind == Encv::KIND => Err(Error::InvalidCombination(
            "encv must wrap a visual sample entry",
        )),
        false => Err(Error::InvalidCombination(
            "enca must wrap an audio sample entry",
        )),
    }
}

/// Split a protected entry's body into its `sinf` and the original entry, which is
/// decoded from the same fields and the remaining child boxes under the type `frma` names.
///
/// `fields` reads the entry's fixed fields (visual or audio), to find where the
/// child boxes start.
fn decode_protected<B: Buf>(
    buf: &mut B,
    kind: FourCC,
    fields: impl FnOnce(&mut &[u8]) -> Result<()>,
) -> Result<(Box<Codec>, Sinf)> {
    let len = buf.remaining();
    let body = buf.slice(len);

    let mut children = body;
    fields(&mut children)?;
    let fields_len = len - children.len();

    // The original entry: a header filled in below, the fields, then every child but `sinf`.
    let mut entry = Vec::with_capacity(len + 8);
    entry.extend_from_slice(&[0; 8]);
    entry.extend_from_slice(&body[..fields_len]);

    let mut sinf = None;
    loop {
        let before = children;
        match Any::decode_maybe(&mut children)? {
            Some(Any::Sinf(atom)) => sinf.insert_once(atom)?,
            Some(_) => entry.extend_from_slice(&before[..before.len() - children.len()]),
            None if children.len() < 8 => {
                // Padding shorter than a box; the original entry tolerates it the same way.
                entry.extend_from_slice(children);
                break;
            }
            // A child whose size runs past the entry.
            None => return Err(Error::OverDecode(kind)),
        }
    }
    let sinf: Sinf = sinf.ok_or(Error::MissingBox(Sinf::KIND))?;

    let original = sinf.frma.data_format;
    if original == Encv::KIND || original == Enca::KIND {
        return Err(Error::InvalidCombination(
            "a protected sample entry cannot wrap another",
        ));
    }

    let size: u32 = entry
        .len()
        .try_into()
        .map_err(|_| Error::TooLarge(original))?;
    entry[..4].copy_from_slice(&size.to_be_bytes());
    entry[4..8].copy_from_slice(&<[u8; 4]>::from(original));

    let codec = Codec::decode(&mut entry.as_slice())?;
    buf.advance(len);

    Ok((Box::new(codec), sinf))
}

/// Write the original entry's body, which must be the type `sinf.frma` names, then `sinf`.
fn encode_protected<B: BufMut>(codec: &Codec, sinf: &Sinf, buf: &mut B) -> Result<()> {
    let mut entry = Vec::new();
    codec.encode(&mut entry)?;

    if entry[4..8] != <[u8; 4]>::from(sinf.frma.data_format) {
        return Err(Error::InvalidCombination(
            "frma must name the protected sample entry's original type",
        ));
    }

    buf.append_slice(&entry[8..]);
    sinf.encode(buf)
}
