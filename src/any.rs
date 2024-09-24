use crate::*;

use std::fmt;
use std::io::Read;

macro_rules! any {
    ($($kind:ident,)*) => {
        /// Any of the supported atoms.
        #[derive(Clone, PartialEq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub enum Any {
            $($kind($kind),)*
			Unknown(FourCC, Bytes),
        }

        impl Any {
            /// Get the kind of the atom.
            pub fn kind(&self) -> FourCC {
                match self {
                    $(Any::$kind(_) => $kind::KIND,)*
					Any::Unknown(kind, _) => *kind,
                }
            }
		}

		impl Decode for Any {
            fn decode(buf: &mut Bytes) -> Result<Self> {
				let header = buf.decode()?;
                Self::decode_atom(&header, buf)
            }
		}

		impl Encode for Any {
            fn encode(&self, buf: &mut BytesMut) -> Result<()> {
				let start = buf.len();
				0u32.encode(buf)?;
				self.kind().encode(buf)?;

				match self {
					$(Any::$kind(inner) => Atom::encode_body(inner, buf),)*
					Any::Unknown(_, data) => data.encode(buf),
				}?;

				let size: u32 = (buf.len() - start).try_into().map_err(|_| Error::TooLarge(self.kind()))?;
				buf[start..start + 4].copy_from_slice(&size.to_be_bytes());

				Ok(())
            }
        }

        impl DecodeAtom for Any {
            /// Decode the atom from a header and payload.
            fn decode_atom(header: &Header, buf: &mut Bytes) -> Result<Self> {
                let size = header.size.unwrap_or(buf.remaining());
                let mut buf: Bytes = buf.decode_exact(size)?;

                let atom = match header.kind {
                    $(_ if header.kind == $kind::KIND => {
                        Any::$kind(match $kind::decode_body(&mut buf) {
                            Ok(atom) => atom,
                            Err(Error::OutOfBounds) => return Err(Error::OverDecode($kind::KIND)),
                            Err(Error::ShortRead) => return Err(Error::UnderDecode($kind::KIND)),
                            Err(err) => return Err(err),
                        })
                    },)*
                    _ => return Ok(Any::Unknown(header.kind, buf.decode()?)),
                };

                if buf.has_remaining() {
                    return Err(Error::UnderDecode(header.kind));
                }

                Ok(atom)
            }
        }

        impl fmt::Debug for Any {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(Any::$kind(inner) => write!(f, "{:?}", inner),)*
                    Any::Unknown(kind, body) => write!(f, "Unknown {{ kind: {:?}, size: {:?} }}", kind, body.len()),
                }
            }
        }

        $(impl From<$kind> for Any {
            fn from(inner: $kind) -> Self {
                Any::$kind(inner)
            }
        })*
    };
}

any! {
    Ftyp,
    Moov,
        Mvhd,
        Udta,
            Meta,
                Ilst,
                    Covr,
                    Desc,
                    Name,
                    Year,
        Trak,
            Tkhd,
            Mdia,
                Mdhd,
                Hdlr,
                Minf,
                    Stbl,
                        Stsd,
                            Avc1,
                                Avcc,
                            Hev1,
                                Hvcc,
                            Mp4a,
                                Esds,
                            Tx3g,
                            Vp09,
                        Stts,
                        Stsc,
                        Stsz,
                        Stss,
                        Stco,
                        Co64,
                        Ctts,
                    Dinf,
                        Dref,
                    Smhd,
                    Vmhd,
            Edts,
                Elst,
        Mvex,
            Mehd,
            Trex,
    Emsg,
    Moof,
        Mfhd,
        Traf,
            Tfhd,
            Tfdt,
            Trun,
    Mdat,
    Free,
}

impl ReadFrom for Any {
    fn read_from<R: Read>(r: &mut R) -> Result<Self> {
        <Option<Any> as ReadFrom>::read_from(r)?.ok_or(Error::UnexpectedEof)
    }
}

impl ReadFrom for Option<Any> {
    fn read_from<R: Read>(r: &mut R) -> Result<Self> {
        let header = match <Option<Header> as ReadFrom>::read_from(r)? {
            Some(header) => header,
            None => return Ok(None),
        };

        let mut buf = header.read_body(r)?;
        Ok(Some(Any::decode_atom(&header, &mut buf)?))
    }
}

impl ReadAtom for Any {
    fn read_atom<R: Read>(header: &Header, r: &mut R) -> Result<Self> {
        let mut buf = header.read_body(r)?;
        Any::decode_atom(header, &mut buf)
    }
}
