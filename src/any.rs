use crate::*;

use std::fmt;
use std::io::Read;

macro_rules! any {
    ($($kind:ident,)*) => {
        #[derive(Clone, PartialEq)]
        pub enum Any {
            $($kind($kind),)*
			Unknown(FourCC, Bytes),
        }

        impl Any {
            pub fn kind(&self) -> FourCC {
                match self {
                    $(Any::$kind(_) => $kind::KIND,)*
					Any::Unknown(kind, _) => *kind,
                }
            }

            pub fn decode_atom(header: Header, buf: &mut Bytes) -> Result<Self> {
                let size = header.size.unwrap_or(buf.remaining());
                let mut buf = buf.decode_exact(size)?;

                let atom = match header.kind {
                    $(_ if header.kind == $kind::KIND => {
                        Any::$kind(match $kind::decode_atom(&mut buf) {
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

		impl Decode for Any {
            fn decode(buf: &mut Bytes) -> Result<Self> {
				let header = buf.decode()?;
                Self::decode_atom(header, buf)
            }
		}

		impl Encode for Any {
            fn encode(&self, buf: &mut BytesMut) -> Result<()> {
				let start = buf.len();
				0u32.encode(buf)?;
				self.kind().encode(buf)?;

				match self {
					$(Any::$kind(inner) => Atom::encode_atom(inner, buf),)*
					Any::Unknown(_, data) => data.encode(buf),
				}?;

				let size: u32 = (buf.len() - start).try_into().map_err(|_| Error::TooLarge(self.kind()))?;
				buf[start..start + 4].copy_from_slice(&size.to_be_bytes());

				Ok(())
            }
        }

        impl ReadFrom for Any {
            fn read_from<R: Read>(r: &mut R) -> Result<Self> {
                Option::<Any>::read_from(r)?.ok_or(Error::UnexpectedEof)
            }
        }

        impl ReadFrom for Option<Any> {
            fn read_from<R: Read>(r: &mut R) -> Result<Self> {
                let header = match Option::<Header>::read_from(r)? {
                    Some(header) => header,
                    None => return Ok(None),
                };

                // TODO This allocates on the heap.
                // Ideally, we should use ReadFrom instead of Decode to avoid this.

                // Don't use `with_capacity` on an untrusted size
                // We allocate at most 4096 bytes upfront and grow as needed
                let cap = header
                    .size
                    .map(|size| std::cmp::max(size, 4096))
                    .unwrap_or(0);

                let mut buf = BytesMut::with_capacity(cap).writer();

                match header.size {
                    Some(size) => {
                        let n = std::io::copy(&mut r.take(size as _), &mut buf)? as _;
                        if size != n {
                            return Err(Error::OutOfBounds);
                        }
                    }
                    None => {
                        std::io::copy(r, &mut buf)?;
                    }
                };


                let mut buf = buf.into_inner().freeze();
                Ok(Some(Any::decode_atom(header, &mut buf)?))
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
