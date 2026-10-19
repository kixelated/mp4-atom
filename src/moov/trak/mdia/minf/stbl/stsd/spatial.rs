use crate::*;

/// The projection and stereo boxes of a visual sample entry. Entries hold it
/// as `spatial: Option<Box<Spatial>>`, since it is large and usually absent.
///
/// Each sample entry writes these boxes just before `pasp`: [Spherical Video
/// V2] requires `st3d` and `sv3d` ahead of the optional `clap` and `pasp`, and
/// ISO/IEC 14496-12 §12.1.4.1 wants `pasp` and `clap` after the boxes of
/// other specifications. `st3d` comes first, as V2 asks, and in any case
/// before `vexu` and `hfov`: a reader may refuse an `st3d` that arrives after
/// another box has set the stereo layout. [Apple] does not constrain where
/// `vexu` and `hfov` go.
///
/// None of these boxes or their children are in [`Any`]. They only mean
/// something inside a sample entry, and several reuse the fourccs of
/// unrelated boxes (two different `proj`, two different `equi`, and ISO's
/// sub-track `stri`). Their containers decode children by header instead.
///
/// A box that repeats, or does not decode, never fails the sample entry: its
/// parent keeps it verbatim, as an [`Any::Unknown`] in `unknown`, and logs a
/// warning. Children this crate does not model are kept there too. On encode,
/// a kept box of a modelled kind goes back at that kind's position, because
/// readers may depend on the order of children.
///
/// A writer may set both families, describing the same projection: a reader
/// that understands both may use either one. The members' docs say what a
/// writer should set.
///
/// [Spherical Video V2]: https://github.com/google/spatial-media/blob/master/docs/spherical-video-v2-rfc.md
/// [Apple]: https://developer.apple.com/av-foundation/Stereo-Video-ISOBMFF-Extensions.pdf
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Spatial {
    pub st3d: Option<St3d>,
    pub sv3d: Option<Sv3d>,
    pub vexu: Option<Vexu>,
    pub hfov: Option<Hfov>,
    /// `st3d`, `sv3d`, `vexu` or `hfov` boxes that repeat or did not decode.
    pub unknown: Vec<Any>,
}

impl Eq for Spatial {}

impl Spatial {
    /// Keep `atom` in `spatial` if it is one of these boxes, creating it on the
    /// first one; otherwise report it as unknown to the sample entry `T`.
    pub(crate) fn decode_child<T: Atom>(
        spatial: &mut Option<Box<Spatial>>,
        atom: Any,
    ) -> Result<()> {
        match atom {
            Any::Unknown(kind @ (St3d::KIND | Sv3d::KIND | Vexu::KIND | Hfov::KIND), body) => {
                let spatial: &mut Spatial = spatial.get_or_insert_with(Default::default);
                let unknown = &mut spatial.unknown;
                match kind {
                    St3d::KIND => decode_slot(T::KIND, &mut spatial.st3d, &body, unknown),
                    Sv3d::KIND => decode_slot(T::KIND, &mut spatial.sv3d, &body, unknown),
                    Vexu::KIND => decode_slot(T::KIND, &mut spatial.vexu, &body, unknown),
                    _ => decode_slot(T::KIND, &mut spatial.hfov, &body, unknown),
                }
                Ok(())
            }
            atom => T::decode_unknown(&atom),
        }
    }
}

impl Encode for Spatial {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.st3d.encode(buf)?;
        encode_kept(&self.unknown, |kind| kind == St3d::KIND, buf)?;
        self.sv3d.encode(buf)?;
        encode_kept(&self.unknown, |kind| kind == Sv3d::KIND, buf)?;
        self.vexu.encode(buf)?;
        encode_kept(&self.unknown, |kind| kind == Vexu::KIND, buf)?;
        self.hfov.encode(buf)?;
        encode_kept(&self.unknown, |kind| kind == Hfov::KIND, buf)?;
        encode_kept(
            &self.unknown,
            |kind| !matches!(kind, St3d::KIND | Sv3d::KIND | Vexu::KIND | Hfov::KIND),
            buf,
        )
    }
}

/// The version and flags of every FullBox here, which their specs define as
/// 0. Anything else is a layout this crate does not know, so the box does not
/// decode and its parent keeps it verbatim.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct FullBoxV0;

impl Ext for FullBoxV0 {
    fn encode(&self) -> Result<u32> {
        Ok(0)
    }

    fn decode(v: u32) -> Result<Self> {
        match v {
            0 => Ok(Self),
            v if v >> 24 != 0 => Err(Error::UnknownVersion((v >> 24) as u8)),
            _ => Err(Error::Unsupported("non-zero FullBox flags")),
        }
    }
}

/// Spherical Video V2: "Parsers should ignore extra bytes at the end of a
/// box", as later revisions may append fields to it.
pub(super) fn ignore_trailing<B: Buf>(kind: FourCC, buf: &mut B) {
    if buf.has_remaining() {
        tracing::warn!(%kind, bytes = buf.remaining(), "ignoring trailing bytes");
        buf.advance(buf.remaining());
    }
}

/// Split a container's body into its children, passing the kind and body of
/// each to `child`.
pub(super) fn decode_children<B: Buf>(
    buf: &mut B,
    mut child: impl FnMut(FourCC, &[u8]),
) -> Result<()> {
    while let Some(header) = Header::decode_maybe(buf)? {
        let size = header.size.unwrap_or(buf.remaining());
        if size > buf.remaining() {
            return Err(Error::OutOfBounds);
        }
        child(header.kind, buf.slice(size));
        buf.advance(size);
    }
    // Tolerate a sub-header padding remainder (e.g. a QuickTime zero terminator).
    skip_trailing_padding(buf);
    Ok(())
}

/// Decode the child `T` from `body` into `slot`, or keep it verbatim in
/// `unknown` if it repeats or does not decode.
pub(super) fn decode_slot<T: Atom>(
    parent: FourCC,
    slot: &mut Option<T>,
    body: &[u8],
    unknown: &mut Vec<Any>,
) {
    decode_variant(parent, slot, body, unknown, |atom: T| atom)
}

/// [`decode_slot`] for a slot that holds one of several boxes, each wrapped
/// by `wrap`.
pub(super) fn decode_variant<T: Atom, S>(
    parent: FourCC,
    slot: &mut Option<S>,
    body: &[u8],
    unknown: &mut Vec<Any>,
    wrap: impl FnOnce(T) -> S,
) {
    let decoded = match slot {
        Some(_) => Err(Error::DuplicateBox(T::KIND)),
        None => decode_exact::<T>(body),
    };
    match decoded {
        Ok(atom) => *slot = Some(wrap(atom)),
        Err(err) => keep_invalid(parent, T::KIND, body, err, unknown),
    }
}

fn decode_exact<T: Atom>(mut body: &[u8]) -> Result<T> {
    let atom = T::decode_body(&mut body)?;
    match body.is_empty() {
        true => Ok(atom),
        false => Err(Error::UnderDecode(T::KIND)),
    }
}

/// Keep a child that could not be used verbatim, with a warning.
pub(super) fn keep_invalid(
    parent: FourCC,
    kind: FourCC,
    body: &[u8],
    err: Error,
    unknown: &mut Vec<Any>,
) {
    tracing::warn!(%kind, %parent, %err, "keeping spatial box verbatim");
    unknown.push(Any::Unknown(kind, body.to_vec()));
}

/// Write the kept children whose type matches `kind`, in their original order.
pub(super) fn encode_kept<B: BufMut>(
    unknown: &[Any],
    kind: impl Fn(FourCC) -> bool,
    buf: &mut B,
) -> Result<()> {
    unknown
        .iter()
        .filter(|atom| kind(atom.kind()))
        .try_for_each(|atom| atom.encode(buf))
}

// Generates `decode_body`/`encode_body` for a container whose children each
// fill one optional field. Each field is written followed by the kept boxes of
// its kind, then the other kept children, then the optional `last` field.
/* example:
spatial_children! { must: Must, stri: Stri, hero: Hero }
spatial_children! { must: Must, eyes: Eyes, pack: Pack; last proj: VexuProj }
*/
macro_rules! spatial_children {
    ($($field:ident: $atom:ident),* $(; last $last:ident: $last_atom:ident)? $(,)?) => {
        fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
            let mut atom = Self::default();
            decode_children(buf, |kind, body| match kind {
                $($atom::KIND => decode_slot(Self::KIND, &mut atom.$field, body, &mut atom.unknown),)*
                $($last_atom::KIND => decode_slot(Self::KIND, &mut atom.$last, body, &mut atom.unknown),)?
                kind => atom.unknown.push(Any::Unknown(kind, body.to_vec())),
            })?;
            Ok(atom)
        }

        fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
            $(
                self.$field.encode(buf)?;
                encode_kept(&self.unknown, |kind| kind == $atom::KIND, buf)?;
            )*
            encode_kept(
                &self.unknown,
                |kind| !matches!(kind, $($atom::KIND)|* $(| $last_atom::KIND)?),
                buf,
            )?;
            $(
                self.$last.encode(buf)?;
                encode_kept(&self.unknown, |kind| kind == $last_atom::KIND, buf)?;
            )?
            Ok(())
        }
    };
}

pub(super) use spatial_children;
