use crate::*;

macro_rules! any {
    ($($kind:ident,)*) => {
        #[derive(Debug, Clone, PartialEq)]
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
		}

		impl Decode for Any {
            fn decode(buf: &mut Bytes) -> Result<Self> {
				let header = Header::decode(buf)?;
				let buf = &mut buf.split_to(header.size.unwrap_or(buf.len()));

                let atom = match header.kind {
					// There's a bug preventing using constants in match arms?
                    $(_ if header.kind == $kind::KIND => Any::$kind(buf.decode()?),)*
					_ => Any::Unknown(header.kind, buf.decode()?),
                };

				if buf.len() > 0 {
					return Err(Error::ShortRead);
				}

				Ok(atom)
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
