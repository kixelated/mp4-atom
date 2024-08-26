use bytes::{Buf, BufMut};

use crate::*;

pub trait Atom: Sized {
    const KIND: FourCC;

    fn decode_inner<B: Buf>(buf: &mut B) -> Result<Self>;
    fn encode_inner<B: BufMut>(&self, buf: &mut B) -> Result<()>;
    fn encode_inner_size(&self) -> usize;
}

impl<T: Atom> Encode for T {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let header = Header {
            kind: Self::KIND,
            size: Some(self.encode_inner_size()),
        };

        header.encode(buf)?;
        header.encode_inner(buf, self)
    }

    fn encode_size(&self) -> usize {
        let header = Header {
            kind: Self::KIND,
            size: Some(self.encode_inner_size()),
        };

        header.encode_size() + header.size.unwrap()
    }
}

impl<T: Atom> Decode for T {
    fn decode<B: Buf>(mut buf: &mut B) -> Result<Self> {
        let header = Header::decode(buf)?;
        header.decode_inner(&mut buf)
    }
}

macro_rules! any {
    ($($kind:ident,)*) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum Any {
            $($kind($kind),)*
			Unknown(Unknown),
        }

        impl Any {
            pub fn kind(&self) -> FourCC {
                match self {
                    $(Any::$kind(_) => $kind::KIND,)*
					Any::Unknown(unknown) => unknown.kind,
                }
            }
		}

		impl Decode for Any {
            fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
				let header = Header::decode(buf)?;

                Ok(match header.kind {
					// There's a bug preventing using constants in match arms?
                    $(_ if header.kind == $kind::KIND => Any::$kind(header.decode_inner(buf)?),)*
					_ => Any::Unknown(Unknown {
						kind: header.kind,
						data: {
							let mut buf = &mut buf.take(header.size.unwrap_or(buf.remaining()));
							buf.decode()?
						},
					}),
                })
            }
		}

		impl Encode for Any {
            fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
				match self {
					$(Any::$kind(inner) => {
						let header = Header {
							kind: $kind::KIND,
							size: Some(inner.encode_inner_size()),
						};

						header.encode(buf)?;
						header.encode_inner(buf, inner)
					},)*
					Any::Unknown(unknown) => unknown.encode(buf),
				}
            }

            fn encode_size(&self) -> usize {
                match self {
                    $(Any::$kind(inner) => {
						let header = Header {
							kind: $kind::KIND,
							size: Some(inner.encode_inner_size()),
						};

						header.encode_size() + inner.encode_inner_size()
					},)*
					Any::Unknown(unknown) => unknown.encode_size(),
                }
            }
        }
    };
}

any! {
    Ftyp,
    Moov,
    Mdat,
    Moof,
    Emsg,
}
