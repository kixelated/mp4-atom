//! # mp4-atom
//!
//! This library provides encoding for the ISO Base Media File Format (ISO/IEC 14496-12).
//! It's meant to be low level, performing encoding/decoding of the binary format without
//! validation or interpretation of the data. You have to know what boxes to expect!

//! ## Atoms
//! MP4 files are made up of atoms, which are boxes of data.
//! They have an upfront size and a FourCC code to identify the type of box.
//! Examples include [Moov], [Mdat], [Trak], etc.
//!
//! Unfortunately, the specification is quite complex and often gated behind a paywall.
//! Using this library does require some additional knowledge of the format otherwise you should use a higher level library.
//!
//! MP4 atoms are often optional and unordered.
//! The simplest way to decode with this library is with [Any::decode], returning any supported atom in a giant enum.
//! For encoding you will call encode on the atom directly, ex: [Moov::encode].
//!
//! ## Traits
//! This library gates functionality behind quite a few traits:
//!
//! - [Atom] is primarily used for encoding/decoding but also provides [Atom::KIND].
//! - [Decode] and [Encode] when using byte slices.
//! - [ReadFrom] and [WriteTo] when using synchronous IO.
//! - **(feature = "tokio")** [AsyncReadFrom] and [AsyncWriteTo] when using asynchronous IO.
//!
//! Additionally, there are some extra traits for decoding atoms given a header.
//! This is useful to avoid decoding large [Mdat] atoms by first reading the header separately.
//!
//! - [DecodeAtom] when using byte slices.
//! - [ReadAtom] when using synchronous IO.
//! - **(feature = "tokio")** [AsyncReadAtom] when using asynchronous IO.
//!
//! There's no equivalent for encoding because the size of the atom is required upfront.
//!
//! ## Examples
//!
//! ### Decoding/encoding a byte buffer
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
//! ### Synchronous IO
//! NOTE: reading a [Mdat] atom will read the entire contents into memory.
//! See the next example to avoid this.
//!
//! ```rust
//! # use bytes::{Buf, BufMut, Bytes, BytesMut};
//! use mp4_atom::{Any, ReadFrom, WriteTo, Ftyp};
//!
//! # fn main() -> anyhow::Result<()> {
//! let mut reader = std::io::stdin();
//!
//! # let mut input = Bytes::from_static(b"\0\0\0\x14ftypiso6\0\0\x02\0mp41");
//! # let mut reader = input.clone().reader(); // Use your own Read type
//!
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
//! let writer = std::io::stdout();
//! # let mut writer = BytesMut::new().writer();
//! atom.write_to(&mut writer)?;
//!
//! # assert_eq!(input, writer.into_inner().freeze());
//! # Ok(()) }
//! ```
//!
//! ### Handling large atoms
//! To avoid reading large files into memory, you can call [Header::read_from] manually:
//!
//! ```rust
//! # use bytes::{Buf, BufMut, Bytes, BytesMut};
//! use mp4_atom::{Atom, Any, Header, ReadFrom, ReadAtom, WriteTo, Ftyp, Moov};
//!
//! # fn main() -> anyhow::Result<()> {
//! let mut reader = std::io::stdin();
//!
//! # let mut input = Bytes::from_static(b"\0\0\0\x14ftypiso6\0\0\x02\0mp41");
//! # let mut reader = input.clone().reader(); // Use your own Read type
//!
//! let header = Header::read_from(&mut reader)?;
//! match header.kind {
//!   Ftyp::KIND => {
//!     let ftyp = Ftyp::read_atom(&header, &mut reader)?;
//!
//!      // Make sure we got the right atom
//!      assert_eq!(ftyp, Ftyp {
//!        major_brand: b"iso6".into(),
//!        minor_version: 512,
//!        compatible_brands: vec![b"mp41".into()],
//!      });
//!    },
//!    Moov::KIND => {
//!      // Manually decode the moov
//!      match header.size {
//!        Some(size) => { /* read size bytes */ },
//!        None => { /* read until EOF */ },
//!      };
//!    },
//!    _ => {
//!      // You can also use Any if you prefer
//!      let any = Any::read_atom(&header, &mut reader)?;
//!      println!("Unknown atom: {:?}", any);
//!    }
//! };
//!
//! # Ok(()) }
//! ```
//!
//! ### Asynchronous IO
//! Enable using the `tokio` feature.
//! It's the same as the above two but using [AsyncReadFrom], [AsyncWriteTo], and [AsyncReadAtom] instead.
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
mod traits;
mod types;

pub use any::*;
pub use atom::*;
pub(crate) use atom_ext::*;
pub use emsg::*;
pub use error::*;
pub use free::*;
pub use ftyp::*;
pub use header::*;
pub use mdat::*;
pub use moof::*;
pub use moov::*;
pub use traits::*;
pub use types::*;

#[cfg(test)]
mod test;
