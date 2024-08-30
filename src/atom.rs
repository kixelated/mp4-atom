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

        let size = header.size.unwrap_or(buf.len());
        if buf.len() < size {
            return Err(Error::OutOfBounds);
        }

        let mut data = buf.split_to(size);
        let atom = match Self::decode_atom(&mut data) {
            Ok(atom) => atom,
            Err(Error::OutOfBounds) => return Err(Error::OverDecode(T::KIND)),
            Err(err) => return Err(err),
        };

        if !data.is_empty() {
            return Err(Error::PartialDecode(T::KIND));
        }

        Ok(atom)
    }
}

impl<T: Atom> ReadFrom for T {
    fn read_from<T: Decode, R: Read>(&mut self, r: R) -> Result<T> {
        let header = Header::read_from(r)?;
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
