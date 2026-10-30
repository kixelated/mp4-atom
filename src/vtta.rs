use crate::*;

/// `VTTAdditionalTextBox` ('vtta'): text of a WebVTT file that isn't cue text,
/// e.g. a comment, as carried in a sample of a WebVTT track (the
/// [`Wvtt`] sample entry).
///
/// See ISO/IEC 14496-30:2018 Section 7.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vtta {
    pub text: String,
}

impl Atom for Vtta {
    const KIND: FourCC = FourCC::new(b"vtta");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Self {
            text: decode_utf8_string(buf)?,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        buf.append_slice(self.text.as_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vtta_roundtrip() {
        let vtta = Vtta {
            text: "NOTE hi".into(),
        };
        let mut buf = Vec::new();
        vtta.encode(&mut buf).unwrap();
        assert_eq!(Vtta::decode(&mut buf.as_slice()).unwrap(), vtta);
    }
}
