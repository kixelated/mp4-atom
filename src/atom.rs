use std::io::Read;

use crate::*;

// A helper to encode/decode a known atom type.
pub trait Atom: Sized {
    const KIND: FourCC;

    fn decode_atom(buf: &mut Bytes) -> Result<Self>;
    fn encode_atom(&self, buf: &mut BytesMut) -> Result<()>;
}

impl<T: Atom> Encode for T {
    #[tracing::instrument(skip_all, fields(?kind = Self::KIND))]
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        let start = buf.len();

        // Encode a 0 for the size, we'll come back to it later
        0u32.encode(buf)?;
        Self::KIND.encode(buf)?;
        self.encode_atom(buf)?;

        // Update the size field
        // TODO support sizes larger than u32 (4GB)
        let size: u32 = (buf.len() - start)
            .try_into()
            .map_err(|_| Error::TooLarge(T::KIND))?;

        buf[start..start + 4].copy_from_slice(&size.to_be_bytes());

        Ok(())
    }
}

impl<T: Atom> Decode for T {
    #[tracing::instrument(skip_all, fields(?kind = Self::KIND))]
    fn decode(buf: &mut Bytes) -> Result<Self> {
        let header = Header::decode(buf)?;

        let size = header.size.unwrap_or(buf.remaining());
        let buf = &mut buf.decode_exact(size)?;

        let atom = match Self::decode_atom(buf) {
            Ok(atom) => atom,
            Err(Error::OutOfBounds) => return Err(Error::OverDecode(T::KIND)),
            Err(Error::ShortRead) => return Err(Error::UnderDecode(T::KIND)),
            Err(err) => return Err(err),
        };

        if buf.has_remaining() {
            return Err(Error::UnderDecode(T::KIND));
        }

        Ok(atom)
    }
}

impl<T: Atom> ReadFrom for T {
    fn read_from<R: Read>(r: &mut R) -> Result<Self> {
        Option::<T>::read_from(r)?.ok_or(Error::MissingBox(T::KIND))
    }
}

impl<T: Atom> ReadFrom for Option<T> {
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

        let buf = &mut buf.into_inner().freeze();

        let atom = match T::decode_atom(buf) {
            Ok(atom) => atom,
            Err(Error::OutOfBounds) => return Err(Error::OverDecode(T::KIND)),
            Err(Error::ShortRead) => return Err(Error::UnderDecode(T::KIND)),
            Err(err) => return Err(err),
        };

        if buf.has_remaining() {
            return Err(Error::UnderDecode(T::KIND));
        }

        Ok(Some(atom))
    }
}

// A helper for generating nested atoms.
/* example:
nested! {
    required: [ Mvhd ],
    optional: [ Meta, Mvex, Udta ],
    multiple: [ Trak ],
};
*/

macro_rules! nested {
    (required: [$($required:ident),*$(,)?], optional: [$($optional:ident),*$(,)?], multiple: [$($multiple:ident),*$(,)?],) => {
        paste::paste! {
            fn decode_atom(buf: &mut Bytes) -> Result<Self> {
                $( let mut [<$required:lower>] = None;)*
                $( let mut [<$optional:lower>] = None;)*
                $( let mut [<$multiple:lower>] = Vec::new();)*

                while let Some(atom) = buf.decode()? {
                    match atom {
                        $(Any::$required(atom) => {
                            if [<$required:lower>].is_some() {
                                return Err(Error::DuplicateBox($required::KIND));
                            }
                            [<$required:lower>] = Some(atom);
                        },)*
                        $(Any::$optional(atom) => {
                            if [<$optional:lower>].is_some() {
                                return Err(Error::DuplicateBox($optional::KIND));
                            }
                            [<$optional:lower>] = Some(atom);
                        },)*
                        $(Any::$multiple(atom) => {
                            [<$multiple:lower>].push(atom);
                        },)*
                        Any::Unknown(kind, _) => {
                            tracing::warn!("unknown box: {:?}", kind);
                        },
                        _ => return Err(Error::UnexpectedBox(atom.kind())),
                    }
                }

                Ok(Self {
                    $([<$required:lower>]: [<$required:lower>].ok_or(Error::MissingBox($required::KIND))? ,)*
                    $([<$optional:lower>],)*
                    $([<$multiple:lower>],)*
                })
            }

            fn encode_atom(&self, buf: &mut BytesMut) -> Result<()> {
                $( self.[<$required:lower>].encode(buf)?; )*
                $( self.[<$optional:lower>].encode(buf)?; )*
                $( self.[<$multiple:lower>].encode(buf)?; )*

                Ok(())
            }
        }
    };
}

pub(crate) use nested;
