//! All ISO-MP4 boxes (atoms) and operations.
//!
//! * [ISO/IEC 14496-12](https://en.wikipedia.org/wiki/MPEG-4_Part_14) - ISO Base Media File Format (QuickTime, MPEG-4, etc)
//! * [ISO/IEC 14496-14](https://en.wikipedia.org/wiki/MPEG-4_Part_14) - MP4 file format
//! * ISO/IEC 14496-17 - Streaming text format
//! * [ISO 23009-1](https://www.iso.org/standard/79329.html) -Dynamic adaptive streaming over HTTP (DASH)
//!
//! http://developer.apple.com/documentation/QuickTime/QTFF/index.html
//! http://www.adobe.com/devnet/video/articles/mp4_movie_atom.html
//! http://mp4ra.org/#/atoms
//!
//! Supported Atoms:
//! ftyp
//! moov
//!     mvhd
//!     udta
//!         meta
//!             ilst
//!                 data
//!     trak
//!         tkhd
//!         mdia
//!             mdhd
//!             hdlr
//!             minf
//!                 stbl
//!                     stsd
//!                         avc1
//!                         hev1
//!                         mp4a
//!                         tx3g
//!                     stts
//!                     stsc
//!                     stsz
//!                     stss
//!                     stco
//!                     co64
//!                     ctts
//!                 dinf
//!                     dref
//!                 smhd
//!                 vmhd
//!         edts
//!             elst
//!     mvex
//!         mehd
//!         trex
//! emsg
//! moof
//!     mfhd
//!     traf
//!         tfhd
//!         tfdt
//!         trun
//! mdat
//! free
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
mod reader;
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
pub use reader::*;
pub use types::*;

#[cfg(test)]
mod test;
