use crate::*;

/// The projection and stereo boxes of a visual sample entry.
///
/// Encoded in the order [Spherical Video V2] requires: `st3d`, then `sv3d`,
/// placed by each sample entry after its configuration box and before the
/// optional boxes. [Apple] does not constrain where `vexu` and `hfov` go.
///
/// [Spherical Video V2]: https://github.com/google/spatial-media/blob/master/docs/spherical-video-v2-rfc.md
/// [Apple]: https://developer.apple.com/av-foundation/Stereo-Video-ISOBMFF-Extensions.pdf
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Spatial {
    pub st3d: Option<St3d>,
    pub sv3d: Option<Sv3d>,
    pub vexu: Option<Vexu>,
    pub hfov: Option<Hfov>,
}

impl Spatial {
    /// Keep `atom` if it is one of these boxes, otherwise report it as unknown
    /// to the sample entry `T`.
    pub(crate) fn decode_child<T: Atom>(&mut self, atom: Any) -> Result<()> {
        match atom {
            Any::St3d(atom) => self.st3d = Some(atom),
            Any::Sv3d(atom) => self.sv3d = Some(atom),
            Any::Vexu(atom) => self.vexu = Some(atom),
            Any::Hfov(atom) => self.hfov = Some(atom),
            unknown => T::decode_unknown(&unknown)?,
        }
        Ok(())
    }
}

impl Encode for Spatial {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.st3d.encode(buf)?;
        self.sv3d.encode(buf)?;
        self.vexu.encode(buf)?;
        self.hfov.encode(buf)
    }
}
