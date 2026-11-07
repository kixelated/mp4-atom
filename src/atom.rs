use std::io::Read;

use crate::*;

/// A helper to encode/decode a known atom type.
pub trait Atom: Sized {
    const KIND: FourCC;

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self>;
    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()>;

    /// Either logs or returns an error depending on the environment/flag.
    fn decode_unknown(atom: &crate::Any) -> Result<()> {
        crate::decode_unknown(atom, Self::KIND)
    }
}

/// The maximum number of atoms that may be nested inside each other.
///
/// Real files nest roughly a dozen levels deep. Untrusted input could
/// otherwise recurse until the stack overflows: a debug build uses up to
/// ~25 KB of stack per level (nested `trak`), so this keeps the worst case
/// under half of a 2 MiB thread stack.
pub(crate) const MAX_DEPTH: usize = 32;

thread_local! {
    static DEPTH: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Tracks the current atom nesting depth, decrementing on drop (including
/// error and unwind paths).
struct DepthGuard;

impl DepthGuard {
    fn enter() -> Result<Self> {
        DEPTH.with(|depth| {
            let next = depth.get() + 1;
            if next > MAX_DEPTH {
                return Err(Error::Unsupported("atom nesting too deep"));
            }
            depth.set(next);
            Ok(DepthGuard)
        })
    }
}

impl Drop for DepthGuard {
    fn drop(&mut self) {
        DEPTH.with(|depth| depth.set(depth.get() - 1));
    }
}

/// Decode an atom body, enforcing the nesting limit and requiring the body to
/// be fully consumed. Every generic body decode should go through here.
pub(crate) fn decode_body_checked<T: Atom, B: Buf>(body: &mut B) -> Result<T> {
    let _guard = DepthGuard::enter()?;

    let atom = match T::decode_body(body) {
        Ok(atom) => atom,
        Err(Error::OutOfBounds) => return Err(Error::OverDecode(T::KIND)),
        Err(Error::ShortRead) => return Err(Error::UnderDecode(T::KIND)),
        Err(err) => return Err(err),
    };

    if body.has_remaining() {
        return Err(Error::UnderDecode(T::KIND));
    }

    Ok(atom)
}

impl<T: Atom> Encode for T {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        // TODO support sizes larger than u32 (4GB)
        buf.encode_atom(Self::KIND, |buf| self.encode_body(buf))
    }
}

impl<T: Atom> Decode for T {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        Self::decode_maybe(buf)?.ok_or(Error::OutOfBounds)
    }
}

impl<T: Atom> DecodeMaybe for T {
    fn decode_maybe<B: Buf>(buf: &mut B) -> Result<Option<Self>> {
        // Decode the header from a view so an incomplete atom leaves the
        // caller's buffer untouched.
        let remaining = buf.remaining();
        let mut peek = buf.slice(remaining);
        let header = match Header::decode_maybe(&mut peek)? {
            Some(header) => header,
            None => return Ok(None),
        };

        if header.kind != T::KIND {
            return Err(Error::UnexpectedBox(header.kind));
        }

        let size = header.size.unwrap_or(peek.remaining());
        if size > peek.remaining() {
            return Ok(None);
        }

        buf.advance(remaining - peek.remaining());
        let body = &mut buf.slice(size);

        let atom = decode_body_checked::<T, _>(body)?;

        buf.advance(size);

        Ok(Some(atom))
    }
}

impl<T: Atom> ReadFrom for T {
    fn read_from<R: Read + ?Sized>(r: &mut R) -> Result<Self> {
        <Option<T> as ReadFrom>::read_from(r)?.ok_or(Error::MissingBox(T::KIND))
    }
}

impl<T: Atom> ReadFrom for Option<T> {
    fn read_from<R: Read + ?Sized>(r: &mut R) -> Result<Self> {
        let header = match <Option<Header> as ReadFrom>::read_from(r)? {
            Some(header) => header,
            None => return Ok(None),
        };

        if header.kind != T::KIND {
            return Err(Error::UnexpectedBox(header.kind));
        }

        let body = &mut header.read_body(r)?;

        let atom = decode_body_checked::<T, _>(body)?;

        Ok(Some(atom))
    }
}

impl<T: Atom> ReadUntil for T {
    fn read_until<R: Read + ?Sized>(r: &mut R) -> Result<Self> {
        <Option<T> as ReadUntil>::read_until(r)?.ok_or(Error::MissingBox(T::KIND))
    }
}

impl<T: Atom> ReadUntil for Option<T> {
    fn read_until<R: Read + ?Sized>(r: &mut R) -> Result<Self> {
        while let Some(header) = <Option<Header> as ReadFrom>::read_from(r)? {
            if header.kind == T::KIND {
                let body = &mut header.read_body(r)?;
                return Ok(Some(T::decode_atom(&header, body)?));
            }

            // Skip the body without buffering it, so the next header is read
            // from after this box rather than from inside it.
            match header.size {
                Some(size) => {
                    let n = std::io::copy(&mut r.take(size as u64), &mut std::io::sink())?;
                    if n != size as u64 {
                        return Err(Error::OutOfBounds);
                    }
                }
                // The box extends to the end of the input, so nothing follows it.
                None => return Ok(None),
            }
        }

        Ok(None)
    }
}

impl<T: Atom> DecodeAtom for T {
    fn decode_atom<B: Buf>(header: &Header, buf: &mut B) -> Result<T> {
        if header.kind != T::KIND {
            return Err(Error::UnexpectedBox(header.kind));
        }

        let size = header.size.unwrap_or(buf.remaining());
        if size > buf.remaining() {
            return Err(Error::OutOfBounds);
        }

        let body = &mut buf.slice(size);

        let atom = decode_body_checked::<T, _>(body)?;

        buf.advance(size);

        Ok(atom)
    }
}

impl<T: Atom> ReadAtom for T {
    fn read_atom<R: Read + ?Sized>(header: &Header, r: &mut R) -> Result<Self> {
        if header.kind != T::KIND {
            return Err(Error::UnexpectedBox(header.kind));
        }

        let body = &mut header.read_body(r)?;
        Self::decode_atom(header, body)
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
        pastey::paste! {
            fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
                $( let mut [<$required:lower>] = None;)*
                $( let mut [<$optional:lower>] = None;)*
                $( let mut [<$multiple:lower>] = Vec::new();)*

                while let Some(atom) = Any::decode_maybe(buf)? {
                    match atom {
                        $(Any::$required(atom) => [<$required:lower>].insert_once(atom)?,)*
                        $(Any::$optional(atom) => [<$optional:lower>].insert_once(atom)?,)*
                        $(Any::$multiple(atom) => {
                            [<$multiple:lower>].push(atom.into());
                        },)*
                        Any::Skip(atom) => tracing::debug!(size = atom.zeroed.size, "skipping skip box"),
                        Any::Free(atom) => tracing::debug!(size = atom.zeroed.size, "skipping free box"),
                        unknown => Self::decode_unknown(&unknown)?,
                    }
                }
                // Tolerate a sub-header padding remainder (e.g. a QuickTime zero
                // terminator) after the child boxes.
                skip_trailing_padding(buf);

                Ok(Self {
                    $([<$required:lower>]: [<$required:lower>].ok_or(Error::MissingBox($required::KIND))? ,)*
                    $([<$optional:lower>],)*
                    $([<$multiple:lower>],)*
                })
            }

            fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
                $( self.[<$required:lower>].encode(buf)?; )*
                $( self.[<$optional:lower>].encode(buf)?; )*
                $( self.[<$multiple:lower>].iter().map(|x| x.encode(buf)).collect::<Result<()>>()?; )*

                Ok(())
            }
        }
    };
}

pub(crate) use nested;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const FTYP_BODY_TAGGED_AS_MOOV: &[u8] = b"\0\0\0\x14mooviso6\0\0\x02\0mp41";

    #[test]
    fn typed_decode_rejects_unexpected_box() {
        let err = Ftyp::decode(&mut Cursor::new(FTYP_BODY_TAGGED_AS_MOOV)).unwrap_err();
        assert!(matches!(err, Error::UnexpectedBox(kind) if kind == Moov::KIND));
    }

    /// Wrap an empty body in `depth` levels of `kind`, each with `prefix`.
    fn nest(kind: &[u8; 4], prefix: &[u8], depth: usize) -> Vec<u8> {
        let mut atom = Vec::new();
        for _ in 0..depth {
            let size = (8 + prefix.len() + atom.len()) as u32;
            let mut outer = Vec::with_capacity(size as usize);
            outer.extend_from_slice(&size.to_be_bytes());
            outer.extend_from_slice(kind);
            outer.extend_from_slice(prefix);
            outer.extend_from_slice(&atom);
            atom = outer;
        }
        atom
    }

    /// Run on a thread with the default 2 MiB test stack, so a regression
    /// aborts with a stack overflow instead of passing on a larger main stack.
    fn on_small_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
        std::thread::Builder::new()
            .stack_size(2 << 20)
            .spawn(f)
            .unwrap()
            .join()
            .unwrap()
    }

    fn is_too_deep<T: std::fmt::Debug>(res: Result<T>) -> bool {
        matches!(res, Err(Error::Unsupported("atom nesting too deep")))
    }

    #[test]
    fn deep_nesting_is_rejected() {
        // Containers with the largest debug stack frames, plus sample
        // entries and stsd which decode their children via Any.
        let containers: &[(&[u8; 4], &[u8])] = &[
            (b"moov", &[]),
            (b"trak", &[]),
            (b"mdia", &[]),
            (b"minf", &[]),
            (b"stbl", &[]),
            (b"traf", &[]),
            (b"ipco", &[]),
            (b"stsd", &[0, 0, 0, 0, 0, 0, 0, 1]),
            (b"avc1", &[0; 78]),
        ];

        for &(kind, prefix) in containers {
            let data = nest(kind, prefix, 5_001);
            let data2 = data.clone();
            assert!(
                on_small_stack(move || is_too_deep(Any::decode(&mut data.as_slice()))),
                "{}",
                FourCC::new(kind)
            );
            assert!(
                on_small_stack(move || is_too_deep(<Any as ReadFrom>::read_from(
                    &mut Cursor::new(data2)
                ))),
                "{}",
                FourCC::new(kind)
            );
        }

        let data = nest(b"moov", &[], 5_001);
        assert!(on_small_stack(move || {
            is_too_deep(Moov::decode(&mut data.as_slice()))
                && is_too_deep(<Moov as ReadFrom>::read_from(&mut Cursor::new(&data)))
        }));
    }

    #[test]
    fn nesting_up_to_limit_decodes() {
        // ipco accepts any child, so nested ipco is valid at any depth.
        on_small_stack(|| {
            let deep = nest(b"ipco", &[], MAX_DEPTH + 1);
            assert!(is_too_deep(Any::decode(&mut deep.as_slice())));

            // The depth counter unwinds after an error, so the same thread
            // can still decode up to the limit.
            let ok = nest(b"ipco", &[], MAX_DEPTH);
            Any::decode(&mut ok.as_slice()).expect("nesting at the limit should decode");
        });
    }

    #[test]
    fn typed_read_from_rejects_unexpected_box() {
        let err =
            <Ftyp as ReadFrom>::read_from(&mut Cursor::new(FTYP_BODY_TAGGED_AS_MOOV)).unwrap_err();
        assert!(matches!(err, Error::UnexpectedBox(kind) if kind == Moov::KIND));
    }

    fn ftyp() -> Ftyp {
        Ftyp {
            major_brand: b"iso6".into(),
            minor_version: 512,
            compatible_brands: vec![b"mp41".into()],
        }
    }

    fn encode(atom: &impl Encode) -> Vec<u8> {
        let mut buf = Vec::new();
        atom.encode(&mut buf).unwrap();
        buf
    }

    #[test]
    fn read_until_ignores_box_nested_in_skipped_body() {
        // A free box whose body happens to contain a valid ftyp.
        let forged = encode(&ftyp());
        let mut input = (8 + forged.len() as u32).to_be_bytes().to_vec();
        input.extend_from_slice(b"free");
        input.extend_from_slice(&forged);

        let found = <Option<Ftyp> as ReadUntil>::read_until(&mut Cursor::new(&input)).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn read_until_finds_box_after_skipped_box() {
        let mut input = encode(&Free {
            zeroed: Zeroed::new(16),
        });
        input.extend(encode(&ftyp()));

        let found = <Ftyp as ReadUntil>::read_until(&mut Cursor::new(&input)).unwrap();
        assert_eq!(found, ftyp());
    }

    #[test]
    fn read_until_rejects_truncated_skipped_box() {
        let mut input = encode(&Free {
            zeroed: Zeroed::new(16),
        });
        input.truncate(input.len() - 4);

        let err = <Ftyp as ReadUntil>::read_until(&mut Cursor::new(&input)).unwrap_err();
        assert!(matches!(err, Error::OutOfBounds));
    }
}
