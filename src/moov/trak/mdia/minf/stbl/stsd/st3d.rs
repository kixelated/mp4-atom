use crate::*;

/// Stereoscopic 3D Video Box, [Spherical Video V2](https://github.com/google/spatial-media/blob/master/docs/spherical-video-v2-rfc.md#stereoscopic-3d-video-box-st3d).
///
/// `stereo_mode` is kept as the wire byte so reserved values round-trip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct St3d {
    pub stereo_mode: u8,
}

impl St3d {
    pub const MONO: u8 = 0;
    /// Left eye on top.
    pub const TOP_BOTTOM: u8 = 1;
    /// Left eye on the left.
    pub const LEFT_RIGHT: u8 = 2;
    /// Packing described by the `mshp` mesh.
    pub const STEREO_CUSTOM: u8 = 3;
}

impl AtomExt for St3d {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"st3d");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        Ok(St3d {
            stereo_mode: u8::decode(buf)?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.stereo_mode.encode(buf)
    }
}
