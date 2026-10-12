use crate::*;

use super::sv3d::skip_child;

/// Video Extended Usage Box, from Apple's [QuickTime and ISO Base Media File
/// Formats and Spatial and Immersive Media][apple]. The other boxes in this
/// file are its children, from the same document.
///
/// [apple]: https://developer.apple.com/av-foundation/Stereo-Video-ISOBMFF-Extensions.pdf
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vexu {
    pub proj: Option<VexuProj>,
    pub eyes: Option<Eyes>,
    pub pack: Option<Pack>,
}

impl Atom for Vexu {
    const KIND: FourCC = FourCC::new(b"vexu");

    // Children are dispatched by header, not through `Any`: `proj` here is not
    // the Google `proj` inside `sv3d`.
    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let mut proj = None;
        let mut eyes = None;
        let mut pack = None;

        while let Some(header) = Header::decode_maybe(buf)? {
            match header.kind {
                VexuProj::KIND => {
                    if proj.is_some() {
                        return Err(Error::DuplicateBox(VexuProj::KIND));
                    }
                    proj = Some(VexuProj::decode_atom(&header, buf)?);
                }
                Eyes::KIND => {
                    if eyes.is_some() {
                        return Err(Error::DuplicateBox(Eyes::KIND));
                    }
                    eyes = Some(Eyes::decode_atom(&header, buf)?);
                }
                Pack::KIND => {
                    if pack.is_some() {
                        return Err(Error::DuplicateBox(Pack::KIND));
                    }
                    pack = Some(Pack::decode_atom(&header, buf)?);
                }
                _ => skip_child::<Self, _>(&header, buf)?,
            }
        }
        skip_trailing_padding(buf);

        Ok(Vexu { proj, eyes, pack })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.proj.encode(buf)?;
        self.eyes.encode(buf)?;
        self.pack.encode(buf)
    }
}

/// The `proj` box inside `vexu`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VexuProj {
    pub prji: Prji,
}

impl Atom for VexuProj {
    const KIND: FourCC = FourCC::new(b"proj");

    nested! {
        required: [ Prji ],
        optional: [],
        multiple: [],
    }
}

/// Projection Information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Prji {
    pub projection: FourCC,
}

impl Prji {
    pub const RECTILINEAR: FourCC = FourCC::new(b"rect");
    pub const EQUIRECTANGULAR: FourCC = FourCC::new(b"equi");
    pub const HALF_EQUIRECTANGULAR: FourCC = FourCC::new(b"hequ");
    pub const FISHEYE: FourCC = FourCC::new(b"fish");
    pub const PARAMETRIC_IMMERSIVE: FourCC = FourCC::new(b"prim");
}

impl AtomExt for Prji {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"prji");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        Ok(Prji {
            projection: FourCC::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.projection.encode(buf)
    }
}

/// Stereo View Box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Eyes {
    pub stri: Option<Stri>,
    pub hero: Option<Hero>,
    pub cams: Option<Cams>,
    pub cmfy: Option<Cmfy>,
}

impl Atom for Eyes {
    const KIND: FourCC = FourCC::new(b"eyes");

    nested! {
        required: [],
        optional: [ Stri, Hero, Cams, Cmfy ],
        multiple: [],
    }
}

/// Stereo View Information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Stri {
    pub eye_views_reversed: bool,
    pub has_additional_views: bool,
    pub has_right_eye_view: bool,
    pub has_left_eye_view: bool,
}

impl AtomExt for Stri {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"stri");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let bits = u8::decode(buf)?;
        Ok(Stri {
            eye_views_reversed: bits & 0b1000 != 0,
            has_additional_views: bits & 0b0100 != 0,
            has_right_eye_view: bits & 0b0010 != 0,
            has_left_eye_view: bits & 0b0001 != 0,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let bits = (self.eye_views_reversed as u8) << 3
            | (self.has_additional_views as u8) << 2
            | (self.has_right_eye_view as u8) << 1
            | (self.has_left_eye_view as u8);
        bits.encode(buf)
    }
}

/// Hero Stereo Eye Description: the eye to show in 2D.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hero {
    pub hero_eye: u8,
}

impl Hero {
    pub const NONE: u8 = 0;
    pub const LEFT: u8 = 1;
    pub const RIGHT: u8 = 2;
}

impl AtomExt for Hero {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"hero");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        Ok(Hero {
            hero_eye: u8::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.hero_eye.encode(buf)
    }
}

/// Stereo Camera System.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cams {
    pub blin: Blin,
}

impl Atom for Cams {
    const KIND: FourCC = FourCC::new(b"cams");

    nested! {
        required: [ Blin ],
        optional: [],
        multiple: [],
    }
}

/// Camera baseline: the distance between the lens centres, in micrometres.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Blin {
    pub baseline: u32,
}

impl AtomExt for Blin {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"blin");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        Ok(Blin {
            baseline: u32::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.baseline.encode(buf)
    }
}

/// Stereo Comfort.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cmfy {
    pub dadj: Dadj,
}

impl Atom for Cmfy {
    const KIND: FourCC = FourCC::new(b"cmfy");

    nested! {
        required: [ Dadj ],
        optional: [],
        multiple: [],
    }
}

/// Horizontal disparity adjustment, in units of 1/10000.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dadj {
    pub disparity_adjustment: i32,
}

impl Dadj {
    pub const DENOMINATOR: i32 = 10_000;
}

impl AtomExt for Dadj {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"dadj");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        Ok(Dadj {
            disparity_adjustment: i32::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.disparity_adjustment.encode(buf)
    }
}

/// Frame Packing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pack {
    pub pkin: Option<Pkin>,
}

impl Atom for Pack {
    const KIND: FourCC = FourCC::new(b"pack");

    nested! {
        required: [],
        optional: [ Pkin ],
        multiple: [],
    }
}

/// Frame Packing Information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pkin {
    pub packing: FourCC,
}

impl Pkin {
    pub const SIDE_BY_SIDE: FourCC = FourCC::new(b"side");
    pub const OVER_UNDER: FourCC = FourCC::new(b"over");
}

impl AtomExt for Pkin {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"pkin");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        Ok(Pkin {
            packing: FourCC::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.packing.encode(buf)
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
    pub horizontal_field_of_view: u32,
}

impl Hfov {
    pub const DENOMINATOR: u32 = 1_000;
}

impl Atom for Hfov {
    const KIND: FourCC = FourCC::new(b"hfov");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Hfov {
            horizontal_field_of_view: u32::decode(buf)?,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.horizontal_field_of_view.encode(buf)
    }
}
