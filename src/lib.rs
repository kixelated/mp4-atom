//! # MP4 / ISO Base Media File Format
//!
//! This library provides encoding for the ISO Base Media File Format (ISO/IEC 14496-12).
//! It's meant to be low level, performing encoding/decoding of the binary format without
//! validation or interpretation of the data. You have to know what boxes to expect!
//!
//! You use the [Encode], [Decode], [ReadFrom], and [WriteTo] traits to encode and decode atoms.
//! The simplest way to use this library is with the [Any] enum, representing one of the supported atoms.
//!
//! ```rust
//! use bytes::{Bytes, BytesMut};
//! use mp4_atom::{Any, Encode, Decode, Ftyp};
//!
//! # fn main() -> anyhow::Result<()> {
//!  // A simple ftyp atom
//! let mut input = Bytes::from_static(b"\0\0\0\x14ftypiso6\0\0\x02\0mp41");
//! let atom = Any::decode(&mut input.clone())?;
//!
//! // Make sure we got the right atom
//! assert_eq!(atom, Ftyp {
//!    major_brand: b"iso6".into(),
//!    minor_version: 512,
//!    compatible_brands: vec![b"mp41".into()],
//! }.into());
//!
//! // Encode it back
//! let mut output = BytesMut::new();
//! atom.encode(&mut output)?;
//!
//! assert_eq!(input, output.freeze());
//! # Ok(()) }
//! ```
//!
//! If you know the type of atom you're expecting, you can use the specific atom type directly.
//!
//! ```rust
//! use bytes::{Bytes, BytesMut};
//! use mp4_atom::{Any, Encode, Decode, Ftyp};
//!
//! # fn main() -> anyhow::Result<()> {
//! let mut input = Bytes::from_static(b"\0\0\0\x14ftypiso6\0\0\x02\0mp41");
//! let atom = Ftyp::decode(&mut input.clone())?;
//!
//! // Make sure we got the right atom
//! assert_eq!(atom, Ftyp {
//!    major_brand: b"iso6".into(),
//!    minor_version: 512,
//!    compatible_brands: vec![b"mp41".into()],
//! }.into());
//!
//! // Encode it back
//! let mut output = BytesMut::new();
//! atom.encode(&mut output)?;
//!
//! assert_eq!(input, output.freeze());
//! # Ok(()) }
//! ```
//!
//! And finally, if you're working with a Reader/Writer, you can use the [ReadFrom] and [WriteTo] traits.
//!
//! ```rust
//! use bytes::{Buf, BufMut, Bytes, BytesMut};
//! use mp4_atom::{Any, ReadFrom, WriteTo, Ftyp};
//!
//! # fn main() -> anyhow::Result<()> {
//! let mut input = Bytes::from_static(b"\0\0\0\x14ftypiso6\0\0\x02\0mp41");
//! let mut reader = input.clone().reader(); // Use your own Read type
//! let atom = Any::read_from(&mut reader)?;
//!
//! // Make sure we got the right atom
//! assert_eq!(atom, Ftyp {
//!    major_brand: b"iso6".into(),
//!    minor_version: 512,
//!    compatible_brands: vec![b"mp41".into()],
//! }.into());
//!
//! // Encode it back to a Write type
//! let mut output = BytesMut::new().writer();
//! atom.write_to(&mut output)?;
//!
//! assert_eq!(input, output.into_inner().freeze());
//! # Ok(()) }
//! ```
//!
//! However, be aware that reading a [Mdat] atom will read the entire contents into memory.
//! If you're working with large files, you may want to call [Header::read_from] first and check the [Header::kind]:
//! - If it's an [Mdat::KIND], then you can read the next [Header::size] bytes manually.
//! - If it's something else, you can use [Header::decode_atom] or [Header::decode_any] like normal.
//!

mod any;
mod atom;
mod atom_ext;
mod coding;
mod emsg;
mod error;
mod free;
mod ftyp;
mod header;
mod mdat;
mod moof;
mod moov;
mod types;

pub use any::*;
pub use atom::*;
pub(crate) use atom_ext::*;
pub use coding::*;
pub use emsg::*;
pub use error::*;
pub use free::*;
pub use ftyp::*;
pub use header::*;
pub use mdat::*;
pub use moof::*;
pub use moov::*;
pub use types::*;

#[cfg(test)]
mod test;
