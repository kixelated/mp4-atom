use crate::*;

use super::spatial::{
    decode_children, decode_slot, decode_variant, encode_kept, ignore_trailing, spatial_children,
};

/// Spherical Video Box, [Spherical Video V2](https://github.com/google/spatial-media/blob/master/docs/spherical-video-v2-rfc.md#spherical-video-box-sv3d).
///
/// The spec requires `svhd` and `proj`; they are optional here, as a parser
/// does not validate. A writer sets both, and `prhd` in `proj`: a reader may
/// expect `svhd`, `proj` and `prhd` in that order, which is how they are
/// written, and ignore the projection when one is missing.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Sv3d {
    pub svhd: Option<Svhd>,
    pub proj: Option<Proj>,
    /// Children this crate does not model, repeats, and children that did not decode.
    pub unknown: Vec<Any>,
}

impl Eq for Sv3d {}

impl Atom for Sv3d {
    const KIND: FourCC = FourCC::new(b"sv3d");

    spatial_children! { svhd: Svhd, proj: Proj }
}

/// Spherical Video Header, [Spherical Video V2](https://github.com/google/spatial-media/blob/master/docs/spherical-video-v2-rfc.md#spherical-video-header-svhd).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Svhd {
    pub metadata_source: String,
}

impl AtomExt for Svhd {
    type Ext = FullBoxV0;

    const KIND_EXT: FourCC = FourCC::new(b"svhd");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: FullBoxV0) -> Result<Self> {
        let metadata_source = String::decode(buf)?;
        ignore_trailing(Self::KIND_EXT, buf);
        Ok(Svhd { metadata_source })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<FullBoxV0> {
        self.metadata_source.as_str().encode(buf)?;
        Ok(FullBoxV0)
    }
}

/// Projection Box, [Spherical Video V2](https://github.com/google/spatial-media/blob/master/docs/spherical-video-v2-rfc.md#projection-box-proj).
///
/// `kind` is `None` when the box holds no projection this crate models; a
/// projection data box of any other type is kept in `unknown`.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Proj {
    pub prhd: Option<Prhd>,
    pub kind: Option<ProjKind>,
    /// Children this crate does not model, repeats, and children that did not decode.
    pub unknown: Vec<Any>,
}

impl Eq for Proj {}

impl Atom for Proj {
    const KIND: FourCC = FourCC::new(b"proj");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let mut proj = Proj::default();
        // "There must not be more than one subclass of a ProjectionDataBox in a
        // given `proj` box", so a second projection is kept as a repeat.
        decode_children(buf, |kind, body| match kind {
            Prhd::KIND => decode_slot(Self::KIND, &mut proj.prhd, body, &mut proj.unknown),
            Equi::KIND => decode_variant(
                Self::KIND,
                &mut proj.kind,
                body,
                &mut proj.unknown,
                ProjKind::Equi,
            ),
            Cbmp::KIND => decode_variant(
                Self::KIND,
                &mut proj.kind,
                body,
                &mut proj.unknown,
                ProjKind::Cbmp,
            ),
            Mshp::KIND => decode_variant(
                Self::KIND,
                &mut proj.kind,
                body,
                &mut proj.unknown,
                ProjKind::Mshp,
            ),
            kind => proj.unknown.push(Any::Unknown(kind, body.to_vec())),
        })?;
        Ok(proj)
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.prhd.encode(buf)?;
        encode_kept(&self.unknown, |kind| kind == Prhd::KIND, buf)?;
        match &self.kind {
            Some(ProjKind::Equi(atom)) => atom.encode(buf)?,
            Some(ProjKind::Cbmp(atom)) => atom.encode(buf)?,
            Some(ProjKind::Mshp(atom)) => atom.encode(buf)?,
            None => {}
        }
        encode_kept(
            &self.unknown,
            |kind| matches!(kind, Equi::KIND | Cbmp::KIND | Mshp::KIND),
            buf,
        )?;
        encode_kept(
            &self.unknown,
            |kind| !matches!(kind, Prhd::KIND | Equi::KIND | Cbmp::KIND | Mshp::KIND),
            buf,
        )
    }
}

/// Projection Header, [Spherical Video V2](https://github.com/google/spatial-media/blob/master/docs/spherical-video-v2-rfc.md#projection-header-box-prhd):
/// the pose, in 16.16 fixed-point degrees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Prhd {
    pub pose_yaw_degrees: i32,
    pub pose_pitch_degrees: i32,
    pub pose_roll_degrees: i32,
}

impl AtomExt for Prhd {
    type Ext = FullBoxV0;

    const KIND_EXT: FourCC = FourCC::new(b"prhd");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: FullBoxV0) -> Result<Self> {
        let prhd = Prhd {
            pose_yaw_degrees: i32::decode(buf)?,
            pose_pitch_degrees: i32::decode(buf)?,
            pose_roll_degrees: i32::decode(buf)?,
        };
        ignore_trailing(Self::KIND_EXT, buf);
        Ok(prhd)
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<FullBoxV0> {
        self.pose_yaw_degrees.encode(buf)?;
        self.pose_pitch_degrees.encode(buf)?;
        self.pose_roll_degrees.encode(buf)?;
        Ok(FullBoxV0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum ProjKind {
    Equi(Equi),
    Cbmp(Cbmp),
    Mshp(Mshp),
}

/// Equirectangular Projection, [Spherical Video V2](https://github.com/google/spatial-media/blob/master/docs/spherical-video-v2-rfc.md#equirectangular-projection-box-equi).
/// The bounds are 0.32 fixed-point fractions of the projection, measured from
/// each edge, that the frame does not cover; all zero for a full 360° frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Equi {
    pub projection_bounds_top: u32,
    pub projection_bounds_bottom: u32,
    pub projection_bounds_left: u32,
    pub projection_bounds_right: u32,
}

impl AtomExt for Equi {
    type Ext = FullBoxV0;

    const KIND_EXT: FourCC = FourCC::new(b"equi");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: FullBoxV0) -> Result<Self> {
        let equi = Equi {
            projection_bounds_top: u32::decode(buf)?,
            projection_bounds_bottom: u32::decode(buf)?,
            projection_bounds_left: u32::decode(buf)?,
            projection_bounds_right: u32::decode(buf)?,
        };
        ignore_trailing(Self::KIND_EXT, buf);
        Ok(equi)
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<FullBoxV0> {
        self.projection_bounds_top.encode(buf)?;
        self.projection_bounds_bottom.encode(buf)?;
        self.projection_bounds_left.encode(buf)?;
        self.projection_bounds_right.encode(buf)?;
        Ok(FullBoxV0)
    }
}

/// Cubemap Projection, [Spherical Video V2](https://github.com/google/spatial-media/blob/master/docs/spherical-video-v2-rfc.md#cubemap-projection-box-cbmp).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cbmp {
    pub layout: u32,
    pub padding: u32,
}

impl AtomExt for Cbmp {
    type Ext = FullBoxV0;

    const KIND_EXT: FourCC = FourCC::new(b"cbmp");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: FullBoxV0) -> Result<Self> {
        let cbmp = Cbmp {
            layout: u32::decode(buf)?,
            padding: u32::decode(buf)?,
        };
        ignore_trailing(Self::KIND_EXT, buf);
        Ok(cbmp)
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<FullBoxV0> {
        self.layout.encode(buf)?;
        self.padding.encode(buf)?;
        Ok(FullBoxV0)
    }
}

/// Mesh Projection, [Spherical Video V2](https://github.com/google/spatial-media/blob/master/docs/spherical-video-v2-rfc.md#mesh-projection-box-mshp).
/// `data` holds the `mesh` boxes and any boxes after them, compressed with raw
/// DEFLATE (RFC 1951, no zlib or gzip header) when `encoding_four_cc` is
/// `dfl8`, or as-is when it is `raw `. `crc` is the CRC-32 of every byte after
/// it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mshp {
    pub crc: u32,
    pub encoding_four_cc: FourCC,
    pub data: Vec<u8>,
}

impl AtomExt for Mshp {
    type Ext = FullBoxV0;

    const KIND_EXT: FourCC = FourCC::new(b"mshp");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: FullBoxV0) -> Result<Self> {
        Ok(Mshp {
            crc: u32::decode(buf)?,
            encoding_four_cc: FourCC::decode(buf)?,
            data: Vec::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<FullBoxV0> {
        self.crc.encode(buf)?;
        self.encoding_four_cc.encode(buf)?;
        self.data.encode(buf)?;
        Ok(FullBoxV0)
    }
}
