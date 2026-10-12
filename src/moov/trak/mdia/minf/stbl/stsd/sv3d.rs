use crate::*;

/// Spherical Video Box, [Spherical Video V2](https://github.com/google/spatial-media/blob/master/docs/spherical-video-v2-rfc.md#spherical-video-box-sv3d).
///
/// The spec requires `svhd` and `proj`; they are optional here so that
/// incomplete metadata does not fail the whole sample entry.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sv3d {
    pub svhd: Option<Svhd>,
    pub proj: Option<Proj>,
}

impl Atom for Sv3d {
    const KIND: FourCC = FourCC::new(b"sv3d");

    // Children are dispatched by header, not through `Any`: Apple's `vexu`
    // has an unrelated `proj` box.
    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let mut svhd = None;
        let mut proj = None;

        while let Some(header) = Header::decode_maybe(buf)? {
            match header.kind {
                Svhd::KIND => {
                    if svhd.is_some() {
                        return Err(Error::DuplicateBox(Svhd::KIND));
                    }
                    svhd = Some(Svhd::decode_atom(&header, buf)?);
                }
                Proj::KIND => {
                    if proj.is_some() {
                        return Err(Error::DuplicateBox(Proj::KIND));
                    }
                    proj = Some(Proj::decode_atom(&header, buf)?);
                }
                _ => skip_child::<Self, _>(&header, buf)?,
            }
        }
        skip_trailing_padding(buf);

        Ok(Sv3d { svhd, proj })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.svhd.encode(buf)?;
        self.proj.encode(buf)
    }
}

/// Spherical Video Header, [Spherical Video V2](https://github.com/google/spatial-media/blob/master/docs/spherical-video-v2-rfc.md#spherical-video-header-svhd).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Svhd {
    pub metadata_source: String,
}

impl AtomExt for Svhd {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"svhd");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        Ok(Svhd {
            metadata_source: String::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.metadata_source.as_str().encode(buf)
    }
}

/// Projection Box, [Spherical Video V2](https://github.com/google/spatial-media/blob/master/docs/spherical-video-v2-rfc.md#projection-box-proj).
///
/// `kind` is `None` when the box holds no projection this crate models.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Proj {
    pub prhd: Option<Prhd>,
    pub kind: Option<ProjKind>,
}

impl Atom for Proj {
    const KIND: FourCC = FourCC::new(b"proj");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let mut prhd = None;
        let mut kind = None;

        while let Some(header) = Header::decode_maybe(buf)? {
            let projection = match header.kind {
                Prhd::KIND => {
                    if prhd.is_some() {
                        return Err(Error::DuplicateBox(Prhd::KIND));
                    }
                    prhd = Some(Prhd::decode_atom(&header, buf)?);
                    continue;
                }
                Equi::KIND => ProjKind::Equi(Equi::decode_atom(&header, buf)?),
                Cbmp::KIND => ProjKind::Cbmp(Cbmp::decode_atom(&header, buf)?),
                Mshp::KIND => ProjKind::Mshp(Mshp::decode_atom(&header, buf)?),
                _ => {
                    skip_child::<Self, _>(&header, buf)?;
                    continue;
                }
            };
            if kind.replace(projection).is_some() {
                return Err(Error::DuplicateBox(header.kind));
            }
        }
        skip_trailing_padding(buf);

        Ok(Proj { prhd, kind })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.prhd.encode(buf)?;
        match &self.kind {
            Some(ProjKind::Equi(atom)) => atom.encode(buf),
            Some(ProjKind::Cbmp(atom)) => atom.encode(buf),
            Some(ProjKind::Mshp(atom)) => atom.encode(buf),
            None => Ok(()),
        }
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
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"prhd");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        Ok(Prhd {
            pose_yaw_degrees: i32::decode(buf)?,
            pose_pitch_degrees: i32::decode(buf)?,
            pose_roll_degrees: i32::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.pose_yaw_degrees.encode(buf)?;
        self.pose_pitch_degrees.encode(buf)?;
        self.pose_roll_degrees.encode(buf)
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
/// The bounds are 0.32 fixed-point fractions of the frame cropped from each edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Equi {
    pub projection_bounds_top: u32,
    pub projection_bounds_bottom: u32,
    pub projection_bounds_left: u32,
    pub projection_bounds_right: u32,
}

impl AtomExt for Equi {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"equi");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        Ok(Equi {
            projection_bounds_top: u32::decode(buf)?,
            projection_bounds_bottom: u32::decode(buf)?,
            projection_bounds_left: u32::decode(buf)?,
            projection_bounds_right: u32::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.projection_bounds_top.encode(buf)?;
        self.projection_bounds_bottom.encode(buf)?;
        self.projection_bounds_left.encode(buf)?;
        self.projection_bounds_right.encode(buf)
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
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"cbmp");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        Ok(Cbmp {
            layout: u32::decode(buf)?,
            padding: u32::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.layout.encode(buf)?;
        self.padding.encode(buf)
    }
}

/// Mesh Projection, [Spherical Video V2](https://github.com/google/spatial-media/blob/master/docs/spherical-video-v2-rfc.md#mesh-projection-box-mshp).
/// `data` holds the `mesh` boxes, deflated when `encoding_four_cc` is `dfl8`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mshp {
    pub crc: u32,
    pub encoding_four_cc: FourCC,
    pub data: Vec<u8>,
}

impl AtomExt for Mshp {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"mshp");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        Ok(Mshp {
            crc: u32::decode(buf)?,
            encoding_four_cc: FourCC::decode(buf)?,
            data: Vec::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.crc.encode(buf)?;
        self.encoding_four_cc.encode(buf)?;
        self.data.encode(buf)
    }
}

/// Skip a child of a container that dispatches by header: `free`/`skip`
/// silently, anything else through `decode_unknown`.
pub(super) fn skip_child<T: Atom, B: Buf>(header: &Header, buf: &mut B) -> Result<()> {
    let size = header.size.unwrap_or(buf.remaining());
    if size > buf.remaining() {
        return Err(Error::OutOfBounds);
    }
    if header.kind != Free::KIND && header.kind != Skip::KIND {
        let body = Vec::decode(&mut buf.slice(size))?;
        T::decode_unknown(&Any::Unknown(header.kind, body))?;
    }
    buf.advance(size);
    Ok(())
}
