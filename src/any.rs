use crate::*;

use std::fmt;

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

            pub fn decode_with(header: Header, buf: &mut Bytes) -> Result<Self> {
                let size = header.size.unwrap_or(buf.len());

                let mut buf = buf.split_to(size);
                let atom = match header.kind {
                    $(_ if header.kind == $kind::KIND => Any::$kind($kind::decode_atom(&mut buf)?),)*
                    _ => return Ok(Any::Unknown(header.kind, buf)),
                };

                if buf.len() > 0 {
                    return Err(Error::ShortRead);
                }

                Ok(atom)
            }
		}

		impl Decode for Any {
            fn decode(buf: &mut Bytes) -> Result<Self> {
				let header = Header::decode(buf)?;
                Self::decode_with(header, buf)
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

				let size: u32 = (buf.len() - start).try_into().map_err(|_| Error::LongWrite)?;
				buf[start..start + 4].copy_from_slice(&size.to_be_bytes());

				Ok(())
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
                            Hev1,
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
