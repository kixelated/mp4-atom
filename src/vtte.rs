use crate::*;

/// `VTTEmptyCueBox` ('vtte'), a sample carried by a WebVTT track (the
/// [`Wvtt`] sample entry) to signal that no cue is active for the
/// enclosing sample's duration.
///
/// See ISO/IEC 14496-30:2018 Section 6.6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vtte;

impl Atom for Vtte {
    const KIND: FourCC = FourCC::new(b"vtte");

    fn decode_body<B: Buf>(_buf: &mut B) -> Result<Self> {
        Ok(Self)
    }

    fn encode_body<B: BufMut>(&self, _buf: &mut B) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vtte_roundtrip() {
        let mut buf = Vec::new();
        Vtte.encode(&mut buf).unwrap();
        assert_eq!(buf, [0, 0, 0, 8, b'v', b't', b't', b'e']);
        assert_eq!(Vtte::decode(&mut buf.as_slice()).unwrap(), Vtte);
    }
}
