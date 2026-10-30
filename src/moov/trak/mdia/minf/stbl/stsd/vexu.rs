use crate::*;

use super::spatial::{decode_children, decode_slot, encode_kept, keep_invalid, spatial_children};

/// Video Extended Usage Box, from Apple's [QuickTime and ISO Base Media File
/// Formats and Spatial and Immersive Media][apple]. The same document defines
/// its children below, and the sibling [`Hfov`].
///
/// Children may come in any order, and the spec expects new ones, so
/// children this crate does not model (such as the lens collection `lnsc`)
/// are kept verbatim in `unknown`, as are repeats and children that do not
/// decode.
///
/// [apple]: https://developer.apple.com/av-foundation/Stereo-Video-ISOBMFF-Extensions.pdf
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Vexu {
    pub must: Option<Must>,
    pub proj: Option<VexuProj>,
    pub eyes: Option<Eyes>,
    pub pack: Option<Pack>,
    pub unknown: Vec<Any>,
}

impl Eq for Vexu {}

impl Atom for Vexu {
    const KIND: FourCC = FourCC::new(b"vexu");

    // Apple does not prescribe an order. `proj` goes last, after any other
    // child: a reader may read only the `prji` at the start of `proj` and take
    // the rest of `proj` for the next sibling. `eyes` goes before `pack`, so a
    // reader applying them in order ends with the packing from `pack`.
    spatial_children! { must: Must, eyes: Eyes, pack: Pack; last proj: VexuProj }
}

/// Required Box Types Box: the types of sibling boxes a reader must
/// understand to use the parent box. Zero entries are reserved.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Must {
    pub required_box_types: Vec<FourCC>,
}

impl AtomExt for Must {
    type Ext = FullBoxV0;

    const KIND_EXT: FourCC = FourCC::new(b"must");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: FullBoxV0) -> Result<Self> {
        let mut required_box_types = Vec::with_capacity(buf.remaining() / 4);
        while buf.remaining() >= 4 {
            required_box_types.push(FourCC::decode(buf)?);
        }
        Ok(Must { required_box_types })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<FullBoxV0> {
        self.required_box_types.encode(buf)?;
        Ok(FullBoxV0)
    }
}

/// The Projection Box inside `vexu`: the projection kind, and optionally the
/// box for that kind. A writer sets `prji`: a reader may refuse a `proj`
/// without one.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct VexuProj {
    pub must: Option<Must>,
    pub prji: Option<Prji>,
    pub kind: Option<VexuProjKind>,
    pub unknown: Vec<Any>,
}

impl Eq for VexuProj {}

impl Atom for VexuProj {
    const KIND: FourCC = FourCC::new(b"proj");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let mut proj = VexuProj::default();
        decode_children(buf, |kind, body| match kind {
            Must::KIND => decode_slot(Self::KIND, &mut proj.must, body, &mut proj.unknown),
            Prji::KIND => decode_slot(Self::KIND, &mut proj.prji, body, &mut proj.unknown),
            kind => match VexuProjKind::from_kind(kind) {
                Some(projection) => {
                    let decoded = match proj.kind {
                        Some(_) => Err(Error::DuplicateBox(kind)),
                        None => VexuProjKind::check_box(kind, body),
                    };
                    match decoded {
                        Ok(()) => proj.kind = Some(projection),
                        Err(err) => keep_invalid(Self::KIND, kind, body, err, &mut proj.unknown),
                    }
                }
                None => proj.unknown.push(Any::Unknown(kind, body.to_vec())),
            },
        })?;
        Ok(proj)
    }

    // `prji` goes first: a reader may refuse a `proj` that starts with
    // anything else.
    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let projection_kind = |kind| VexuProjKind::from_kind(kind).is_some();
        self.prji.encode(buf)?;
        encode_kept(&self.unknown, |kind| kind == Prji::KIND, buf)?;
        self.kind.encode(buf)?;
        encode_kept(&self.unknown, projection_kind, buf)?;
        self.must.encode(buf)?;
        encode_kept(&self.unknown, |kind| kind == Must::KIND, buf)?;
        encode_kept(
            &self.unknown,
            |kind| !matches!(kind, Prji::KIND | Must::KIND) && !projection_kind(kind),
            buf,
        )
    }
}

/// Projection Information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Prji {
    pub projection_kind: FourCC,
}

impl Prji {
    pub const RECTILINEAR: FourCC = FourCC::new(b"rect");
    pub const EQUIRECTANGULAR: FourCC = FourCC::new(b"equi");
    pub const HALF_EQUIRECTANGULAR: FourCC = FourCC::new(b"hequ");
    pub const FISHEYE: FourCC = FourCC::new(b"fish");
    pub const PARAMETRIC_IMMERSIVE: FourCC = FourCC::new(b"prim");
}

impl AtomExt for Prji {
    type Ext = FullBoxV0;

    const KIND_EXT: FourCC = FourCC::new(b"prji");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: FullBoxV0) -> Result<Self> {
        Ok(Prji {
            projection_kind: FourCC::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<FullBoxV0> {
        self.projection_kind.encode(buf)?;
        Ok(FullBoxV0)
    }
}

/// The box for a projection kind, a sibling of `prji` whose type matches its
/// `projection_kind`. None of these boxes defines any fields yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum VexuProjKind {
    /// RectilinearProjectionBox, `rect`.
    Rectilinear,
    /// FullEquirectangularProjectionBox, `equi`.
    Equirectangular,
    /// HalfEquirectangularProjectionBox, `hequ`.
    HalfEquirectangular,
    /// FisheyeProjectionBox, `fish`.
    Fisheye,
    /// ParametricImmersiveMediaProjectionBox, `prim`.
    ParametricImmersive,
}

impl VexuProjKind {
    /// The box type, which is also the `projection_kind` it goes with.
    pub fn kind(self) -> FourCC {
        match self {
            Self::Rectilinear => Prji::RECTILINEAR,
            Self::Equirectangular => Prji::EQUIRECTANGULAR,
            Self::HalfEquirectangular => Prji::HALF_EQUIRECTANGULAR,
            Self::Fisheye => Prji::FISHEYE,
            Self::ParametricImmersive => Prji::PARAMETRIC_IMMERSIVE,
        }
    }

    fn from_kind(kind: FourCC) -> Option<Self> {
        Some(match kind {
            Prji::RECTILINEAR => Self::Rectilinear,
            Prji::EQUIRECTANGULAR => Self::Equirectangular,
            Prji::HALF_EQUIRECTANGULAR => Self::HalfEquirectangular,
            Prji::FISHEYE => Self::Fisheye,
            Prji::PARAMETRIC_IMMERSIVE => Self::ParametricImmersive,
            _ => return None,
        })
    }

    /// Each kind is a FullBox with version and flags 0 and no fields.
    fn check_box(kind: FourCC, mut body: &[u8]) -> Result<()> {
        FullBoxV0::decode(u32::decode(&mut body)?)?;
        match body.is_empty() {
            true => Ok(()),
            false => Err(Error::UnderDecode(kind)),
        }
    }
}

impl Encode for VexuProjKind {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        buf.encode_atom_value(self.kind(), &0u32)
    }
}

/// Stereo View Box. The spec requires `stri`; it is optional here, as a
/// parser does not validate, but a writer sets it.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Eyes {
    pub must: Option<Must>,
    pub stri: Option<Stri>,
    pub hero: Option<Hero>,
    pub cams: Option<Cams>,
    pub cmfy: Option<Cmfy>,
    pub unknown: Vec<Any>,
}

impl Eq for Eyes {}

impl Atom for Eyes {
    const KIND: FourCC = FourCC::new(b"eyes");

    spatial_children! { must: Must, stri: Stri, hero: Hero, cams: Cams, cmfy: Cmfy }
}

/// Stereo View Information. Readers must treat a box with non-zero reserved
/// bits as invalid, so such a box does not decode and is kept verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Stri {
    pub eye_views_reversed: bool,
    pub has_additional_views: bool,
    pub has_right_eye_view: bool,
    pub has_left_eye_view: bool,
}

impl AtomExt for Stri {
    type Ext = FullBoxV0;

    const KIND_EXT: FourCC = FourCC::new(b"stri");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: FullBoxV0) -> Result<Self> {
        let bits = u8::decode(buf)?;
        if bits & 0xf0 != 0 {
            return Err(Error::Reserved);
        }
        Ok(Stri {
            eye_views_reversed: bits & 0b1000 != 0,
            has_additional_views: bits & 0b0100 != 0,
            has_right_eye_view: bits & 0b0010 != 0,
            has_left_eye_view: bits & 0b0001 != 0,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<FullBoxV0> {
        let bits = (self.eye_views_reversed as u8) << 3
            | (self.has_additional_views as u8) << 2
            | (self.has_right_eye_view as u8) << 1
            | (self.has_left_eye_view as u8);
        bits.encode(buf)?;
        Ok(FullBoxV0)
    }
}

/// Hero Stereo Eye Description: the eye to show in 2D. Values of 3 and above
/// are reserved, and readers treat them as no hero eye.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hero {
    pub hero_eye_indicator: u8,
}

impl Hero {
    pub const NONE: u8 = 0;
    pub const LEFT: u8 = 1;
    pub const RIGHT: u8 = 2;
}

impl AtomExt for Hero {
    type Ext = FullBoxV0;

    const KIND_EXT: FourCC = FourCC::new(b"hero");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: FullBoxV0) -> Result<Self> {
        Ok(Hero {
            hero_eye_indicator: u8::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<FullBoxV0> {
        self.hero_eye_indicator.encode(buf)?;
        Ok(FullBoxV0)
    }
}

/// Stereo Camera System. A reader may refuse a `cams` that holds anything
/// besides a single `blin`, so a writer leaves `must` and `unknown` empty and
/// does not write an empty `cams`.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Cams {
    pub must: Option<Must>,
    pub blin: Option<Blin>,
    pub unknown: Vec<Any>,
}

impl Eq for Cams {}

impl Atom for Cams {
    const KIND: FourCC = FourCC::new(b"cams");

    spatial_children! { must: Must, blin: Blin }
}

/// Stereo Camera System Baseline: the distance between the lens centres, in
/// micrometres.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Blin {
    pub baseline_value: u32,
}

impl AtomExt for Blin {
    type Ext = FullBoxV0;

    const KIND_EXT: FourCC = FourCC::new(b"blin");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: FullBoxV0) -> Result<Self> {
        Ok(Blin {
            baseline_value: u32::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<FullBoxV0> {
        self.baseline_value.encode(buf)?;
        Ok(FullBoxV0)
    }
}

/// Stereo Comfort. A reader may refuse a `cmfy` that holds anything besides a
/// single `dadj`, so a writer leaves `must` and `unknown` empty and does not
/// write an empty `cmfy`.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Cmfy {
    pub must: Option<Must>,
    pub dadj: Option<Dadj>,
    pub unknown: Vec<Any>,
}

impl Eq for Cmfy {}

impl Atom for Cmfy {
    const KIND: FourCC = FourCC::new(b"cmfy");

    spatial_children! { must: Must, dadj: Dadj }
}

/// Stereo Comfort Disparity Adjustment, in 1/10000 of the width of one eye's
/// view (-10000 to 10000). Half of it is added to the left eye and half
/// subtracted from the right.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dadj {
    pub disparity_adjustment: i32,
}

impl Dadj {
    pub const DENOMINATOR: i32 = 10_000;
}

impl AtomExt for Dadj {
    type Ext = FullBoxV0;

    const KIND_EXT: FourCC = FourCC::new(b"dadj");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: FullBoxV0) -> Result<Self> {
        Ok(Dadj {
            disparity_adjustment: i32::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<FullBoxV0> {
        self.disparity_adjustment.encode(buf)?;
        Ok(FullBoxV0)
    }
}

/// View Packing.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Pack {
    pub must: Option<Must>,
    pub pkin: Option<Pkin>,
    pub unknown: Vec<Any>,
}

impl Eq for Pack {}

impl Atom for Pack {
    const KIND: FourCC = FourCC::new(b"pack");

    spatial_children! { must: Must, pkin: Pkin }
}

/// View Packing Information. A `view_packing_kind` of zero is a placeholder
/// for a packing set later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pkin {
    pub view_packing_kind: FourCC,
}

impl Pkin {
    pub const SIDE_BY_SIDE: FourCC = FourCC::new(b"side");
    pub const OVER_UNDER: FourCC = FourCC::new(b"over");
}

impl AtomExt for Pkin {
    type Ext = FullBoxV0;

    const KIND_EXT: FourCC = FourCC::new(b"pkin");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: FullBoxV0) -> Result<Self> {
        Ok(Pkin {
            view_packing_kind: FourCC::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<FullBoxV0> {
        self.view_packing_kind.encode(buf)?;
        Ok(FullBoxV0)
    }
}

/// Horizontal Field of View Box, from Apple's [QuickTime and ISO Base Media
/// File Formats and Spatial and Immersive Media][apple], in units of 1/1000
/// degree. A sibling of `vexu` in the sample entry.
///
/// [apple]: https://developer.apple.com/av-foundation/Stereo-Video-ISOBMFF-Extensions.pdf
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hfov {
    pub field_of_view: u32,
}

impl Hfov {
    pub const DENOMINATOR: u32 = 1_000;
}

impl Atom for Hfov {
    const KIND: FourCC = FourCC::new(b"hfov");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Hfov {
            field_of_view: u32::decode(buf)?,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.field_of_view.encode(buf)
    }
}
