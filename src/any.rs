use crate::*;

macro_rules! any {
    ($($kind:ident,)*) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum Any {
            $($kind($kind),)*
			Unknown(FourCC, Vec<u8>),
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
            fn decode(buf: &mut Buf) -> Result<Self> {
				let header = Header::decode(buf)?;
				let buf = &mut buf.take(header.size.unwrap_or(buf.len()))?;

                Ok(match header.kind {
					// There's a bug preventing using constants in match arms?
                    $(_ if header.kind == $kind::KIND => Any::$kind(buf.decode()?),)*
					_ => Any::Unknown(header.kind, buf.decode()?),
                })
            }
		}

		impl Encode for Any {
            fn encode(&self, buf: &mut BufMut) -> Result<()> {
				let start = buf.len();
				0u32.encode(buf)?;
				self.kind().encode(buf)?;

				match self {
					$(Any::$kind(inner) => inner.encode_inner(buf),)*
					Any::Unknown(_, data) => data.encode(buf),
				}?;

				let size = (buf.len() - start).try_into().map_err(|_| Error::LongWrite)?;
				buf.u32_at(size, start)
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
                    Data,
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
                            Tx3g,
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
