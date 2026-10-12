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
            Any::St3d(atom) => set_once(&mut self.st3d, atom),
            Any::Sv3d(atom) => set_once(&mut self.sv3d, atom),
            Any::Vexu(atom) => set_once(&mut self.vexu, atom),
            Any::Hfov(atom) => set_once(&mut self.hfov, atom),
            unknown => T::decode_unknown(&unknown),
        }
    }
}

/// Each of these boxes may appear at most once in a sample entry.
fn set_once<T: Atom>(slot: &mut Option<T>, atom: T) -> Result<()> {
    match slot.replace(atom) {
        Some(_) => Err(Error::DuplicateBox(T::KIND)),
        None => Ok(()),
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
