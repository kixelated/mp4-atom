use crate::*;

const IDEN: FourCC = FourCC::new(b"iden");
const CTIM: FourCC = FourCC::new(b"ctim");
const STTG: FourCC = FourCC::new(b"sttg");
const PAYL: FourCC = FourCC::new(b"payl");

/// `VTTCueBox` ('vttc'): one WebVTT cue, as carried in a sample of a WebVTT
/// track (the [`Wvtt`] sample entry). Like [`Emib`], it is sample
/// content rather than a structural box, so it isn't registered in [`Any`].
///
/// See ISO/IEC 14496-30:2018 Section 7.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vttc {
    /// `CueSourceIDBox` ('iden'): the cue identifier.
    pub iden: Option<String>,
    /// `CueTimeBox` ('ctim'): the original start time, if the cue was split.
    pub ctim: Option<String>,
    /// `CueSettingsBox` ('sttg'): the cue settings.
    pub sttg: Option<String>,
    /// `CuePayloadBox` ('payl'): the cue text.
    pub payl: String,
}

impl Atom for Vttc {
    const KIND: FourCC = FourCC::new(b"vttc");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let mut cue = Self::default();
        let mut payl = None;

        while let Some(header) = Header::decode_maybe(buf)? {
            let destination = match header.kind {
                IDEN => &mut cue.iden,
                CTIM => &mut cue.ctim,
                STTG => &mut cue.sttg,
                PAYL => &mut payl,
                kind => return Err(Error::UnexpectedBox(kind)),
            };
            if destination.is_some() {
                return Err(Error::DuplicateBox(header.kind));
            }
            *destination = Some(header.decode_utf8_string(buf)?);
        }

        cue.payl = payl.ok_or(Error::MissingBox(PAYL))?;
        Ok(cue)
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        if let Some(iden) = &self.iden {
            buf.encode_atom_string(IDEN, iden)?;
        }
        if let Some(ctim) = &self.ctim {
            buf.encode_atom_string(CTIM, ctim)?;
        }
        if let Some(sttg) = &self.sttg {
            buf.encode_atom_string(STTG, sttg)?;
        }
        buf.encode_atom_string(PAYL, &self.payl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vttc_roundtrip() {
        let cue = Vttc {
            iden: Some("1".into()),
            ctim: Some("00:00:01.000".into()),
            sttg: Some("align:start".into()),
            payl: "Hello".into(),
        };
        let mut buf = Vec::new();
        cue.encode(&mut buf).unwrap();
        assert_eq!(Vttc::decode(&mut buf.as_slice()).unwrap(), cue);
    }

    #[test]
    fn test_vttc_missing_payload() {
        let mut buf = Vec::new();
        buf.encode_atom(Vttc::KIND, |buf| buf.encode_atom_string(IDEN, "1"))
            .unwrap();
        let err = Vttc::decode(&mut buf.as_slice()).unwrap_err();
        assert!(matches!(err, Error::MissingBox(kind) if kind == PAYL));
    }
}
