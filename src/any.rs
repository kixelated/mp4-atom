use crate::*;

use std::fmt;
use std::io::Read;

macro_rules! any {
    (basic: [$($kind:ident,)* $(,)?], boxed: [$($boxed:ident,)* $(,)?]) => {
        /// Any of the supported atoms.
        #[derive(Clone, PartialEq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[non_exhaustive]
        pub enum Any {
            $($kind($kind),)*
            $($boxed(Box<$boxed>),)*
            Unknown(FourCC, Vec<u8>),
        }

        impl Any {
            /// Get the kind of the atom.
            pub fn kind(&self) -> FourCC {
                match self {
                    $(Any::$kind(_) => $kind::KIND,)*
                    $(Any::$boxed(_) => $boxed::KIND,)*
                    Any::Unknown(kind, _) => *kind,
                }
            }
        }

        impl Decode for Any {
            fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
                match Self::decode_maybe(buf)? {
                    Some(any) => Ok(any),
                    None => Err(Error::OutOfBounds),
                }
            }
        }

        impl DecodeMaybe for Any {
            fn decode_maybe<B: Buf>(buf: &mut B) -> Result<Option<Self>> {
                let header = match Header::decode_maybe(buf)? {
                    Some(header) => header,
                    None => return Ok(None),
                };

                let size = header.size.unwrap_or(buf.remaining());
                if size > buf.remaining() {
                    return Ok(None);
                }

                Ok(Some(Self::decode_atom(&header, buf)?))
            }
        }

        impl Encode for Any {
            fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
                let start = buf.len();
                0u32.encode(buf)?;
                self.kind().encode(buf)?;

                match self {
                    $(Any::$kind(inner) => Atom::encode_body(inner, buf),)*
                    $(Any::$boxed(boxed) => Atom::encode_body(boxed.as_ref(), buf),)*
                    Any::Unknown(_, data) => data.encode(buf),
                }?;

                let size: u32 = (buf.len() - start).try_into().map_err(|_| Error::TooLarge(self.kind()))?;
                buf.set_slice(start, &size.to_be_bytes());

                Ok(())
            }
        }

        impl DecodeAtom for Any {
            /// Decode the atom from a header and payload.
            fn decode_atom<B: Buf>(header: &Header, buf: &mut B) -> Result<Self> {
                let size = header.size.unwrap_or(buf.remaining());
                if size > buf.remaining() {
                    return Err(Error::OutOfBounds);
                }

                let mut body = &mut buf.slice(size);

                let atom = match header.kind {
                    $(_ if header.kind == $kind::KIND => {
                        Any::$kind(match $kind::decode_body(&mut body) {
                            Ok(atom) => atom,
                            Err(Error::OutOfBounds) => return Err(Error::OverDecode($kind::KIND)),
                            Err(Error::ShortRead) => return Err(Error::UnderDecode($kind::KIND)),
                            Err(err) => return Err(err),
                        })
                    },)*
                    $(_ if header.kind == $boxed::KIND => {
                        Any::$boxed(match $boxed::decode_body(&mut body) {
                            Ok(atom) => Box::new(atom),
                            Err(Error::OutOfBounds) => return Err(Error::OverDecode($boxed::KIND)),
                            Err(Error::ShortRead) => return Err(Error::UnderDecode($boxed::KIND)),
                            Err(err) => return Err(err),
                        })
                    },)*
                    _ => {
                        let body = Vec::decode(body)?;
                        Any::Unknown(header.kind, body)
                    },
                };

                if body.has_remaining() {
                    return Err(Error::UnderDecode(header.kind));
                }

                buf.advance(size);

                Ok(atom)
            }
        }

        impl fmt::Debug for Any {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(Any::$kind(inner) => inner.fmt(f),)*
                    $(Any::$boxed(boxed) => boxed.fmt(f),)*
                    Any::Unknown(kind, body) => write!(f, "Unknown {{ kind: {:?}, size: {:?}, bytes: {:?} }}", kind, body.len(), body),
                }
            }
        }

        $(impl From<$kind> for Any {
            fn from(inner: $kind) -> Self {
                Any::$kind(inner)
            }
        })*

        $(impl From<$boxed> for Any {
            fn from(inner: $boxed) -> Self {
                Any::$boxed(Box::new(inner))
            }
        })*

        $(impl TryFrom<Any> for $kind {
            type Error = Any;

            fn try_from(any: Any) -> std::result::Result<Self, Any> {
                match any {
                    Any::$kind(inner) => Ok(inner),
                    _ => Err(any),
                }
            }
        })*

        $(impl TryFrom<Any> for $boxed {
            type Error = Any;

            fn try_from(any: Any) -> std::result::Result<Self, Any> {
                match any {
                    Any::$boxed(boxed) => Ok(*boxed),
                    _ => Err(any),
                }
            }
        })*

        // So we can use .into() to automatically unbox
        $(impl From<Box<$boxed>> for $boxed {
            fn from(boxed: Box<$boxed>) -> Self {
                *boxed
            }
        })*

        /// A trait to help casting to/from Any.
        /// From/TryFrom use concrete types, but if we want to use generics, then we need a trait.
        pub trait AnyAtom: Atom {
            fn from_any(any: Any) -> Option<Self>;
            fn from_any_ref(any: &Any) -> Option<&Self>;
            fn from_any_mut(any: &mut Any) -> Option<&mut Self>;

            fn into_any(self) -> Any;
        }

        $(impl AnyAtom for $kind {
            fn from_any(any: Any) -> Option<Self> {
                match any {
                    Any::$kind(inner) => Some(inner),
                    _ => None,
                }
            }

            fn from_any_ref(any: &Any) -> Option<&Self> {
                match any {
                    Any::$kind(inner) => Some(inner),
                    _ => None,
                }
            }

            fn from_any_mut(any: &mut Any) -> Option<&mut Self> {
                match any {
                    Any::$kind(inner) => Some(inner),
                    _ => None,
                }
            }

            fn into_any(self) -> Any {
                Any::$kind(self)
            }
        })*

        $(impl AnyAtom for $boxed {
            fn from_any(any: Any) -> Option<Self> {
                match any {
                    Any::$boxed(boxed) => Some(*boxed),
                    _ => None,
                }
            }

            fn from_any_ref(any: &Any) -> Option<&Self> {
                match any {
                    Any::$boxed(boxed) => Some(boxed),
                    _ => None,
                }
            }

            fn from_any_mut(any: &mut Any) -> Option<&mut Self> {
                match any {
                    Any::$boxed(boxed) => Some(boxed),
                    _ => None,
                }
            }

            fn into_any(self) -> Any {
                Any::$boxed(Box::new(self))
            }
        })*
    };
}

any! {
    basic: [
    Ftyp,
    Styp,
    Meta,
        Hdlr,
        Pitm,
        Iloc,
        Iinf,
        Iprp,
            Ipco,
                Auxc,
                Clap,
                Imir,
                Irot,
                Iscl,
                Ispe,
                Pixi,
                Rref,
            Ipma,
        Iref,
        Idat,
        Ilst,
            Covr,
            Desc,
            Tool, // "©too"
            Name,
            Year,
    Moov,
        Mvhd,
        Udta,
            Skip,
        // Trak, // boxed to avoid large size differences between variants
            Tkhd,
            Mdia,
                Mdhd,
                Minf,
                    Stbl,
                        Stsd,
                            Avc1,
                                Avcc,
                                Btrt,
                                Ccst,
                                Colr,
                                Pasp,
                                Taic,
                                Fiel,
                            Hev1, Hvc1,
                                Hvcc,
                            Mp4a,
                                Esds,
                            Tx3g,
                            Vp08, Vp09,
                                VpcC,
                            Av01,
                                Av1c,
                            Opus,
                                Dops,
                            Uncv,
                                Cmpd,
                                UncC,
                            Flac,
                                Dfla,
                            Ac3,
                                Ac3SpecificBox,
                            Eac3,
                                Ec3SpecificBox,
                            Sowt, Twos, Lpcm, Ipcm, Fpcm, In24, In32, Fl32, Fl64, S16l,
                                PcmC,
                                Chnl,
                        Stts,
                        Stsc,
                        Stsz,
                        Stss,
                        Stco,
                        Co64,
                        Cslg,
                        Ctts,
                        Sbgp,
                        Sgpd,
                        Subs,
                        Saio,
                        Saiz,
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
    Prft,
    ],
    boxed: [
        Trak,
    ]
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

        let body = &mut header.read_body(r)?;
        Ok(Some(Any::decode_atom(&header, body)?))
    }
}

impl ReadAtom for Any {
    fn read_atom<R: Read>(header: &Header, r: &mut R) -> Result<Self> {
        let body = &mut header.read_body(r)?;
        Any::decode_atom(header, body)
    }
}
