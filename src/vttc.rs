use crate::*;

const VSID: FourCC = FourCC::new(b"vsid");
const IDEN: FourCC = FourCC::new(b"iden");
const CTIM: FourCC = FourCC::new(b"ctim");
const STTG: FourCC = FourCC::new(b"sttg");
const PAYL: FourCC = FourCC::new(b"payl");

/// `VTTCueBox` ('vttc'): one WebVTT cue, as carried in a sample of a WebVTT
/// track (the [`Wvtt`] sample entry). Like [`Emib`], it is sample
/// content rather than a structural box, so it isn't registered in [`Any`].
///
/// See ISO/IEC 14496-30:2018 Section 6.6.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vttc {
    /// `CueSourceIDBox` ('vsid'): the ID of the source (e.g. WebVTT file) the
    /// cue comes from.
    pub vsid: Option<u32>,
    /// `CueIDBox` ('iden'): the cue identifier.
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
            match header.kind {
                VSID => cue.vsid.insert_if_none_kind(
                    VSID,
                    header.decode_body(buf, |body| u32::decode(body))?,
                )?,
                IDEN => cue
                    .iden
                    .insert_if_none_kind(IDEN, header.decode_utf8_string(buf)?)?,
                CTIM => cue
                    .ctim
                    .insert_if_none_kind(CTIM, header.decode_utf8_string(buf)?)?,
                STTG => cue
                    .sttg
                    .insert_if_none_kind(STTG, header.decode_utf8_string(buf)?)?,
                PAYL => payl.insert_if_none_kind(PAYL, header.decode_utf8_string(buf)?)?,
                // Other boxes are ignored.
                _ => header.skip_body(buf)?,
            }
        }

        cue.payl = payl.ok_or(Error::MissingBox(PAYL))?;
        Ok(cue)
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        if let Some(vsid) = &self.vsid {
            buf.encode_atom(VSID, |buf| vsid.encode(buf))?;
        }
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
            vsid: Some(7),
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
    fn test_vttc_duplicate_box_is_error() {
        for kind in [VSID, IDEN, CTIM, STTG, PAYL] {
            let mut buf = Vec::new();
            buf.encode_atom(Vttc::KIND, |buf| {
                // Every box has a valid body for each of the types.
                for _ in 0..2 {
                    buf.encode_atom(kind, |buf| 1u32.encode(buf))?;
                }
                if kind != PAYL {
                    buf.encode_atom_string(PAYL, "Hi")?;
                }
                Ok(())
            })
            .unwrap();
            let err = Vttc::decode(&mut buf.as_slice()).unwrap_err();
            assert!(
                matches!(err, Error::DuplicateBox(k) if k == kind),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn test_vttc_ignores_other_boxes() {
        let mut buf = Vec::new();
        buf.encode_atom(Vttc::KIND, |buf| {
            buf.encode_atom_string(b"xxxx", "ignored")?;
            buf.encode_atom_string(PAYL, "Hi")?;
            buf.encode_atom_string(b"yyyy", "also ignored")
        })
        .unwrap();
        assert_eq!(
            Vttc::decode(&mut buf.as_slice()).unwrap(),
            Vttc {
                payl: "Hi".into(),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_vttc_vsid() {
        let mut buf = Vec::new();
        buf.encode_atom(Vttc::KIND, |buf| {
            buf.encode_atom(VSID, |buf| 0x01020304u32.encode(buf))?;
            buf.encode_atom_string(PAYL, "Hi")
        })
        .unwrap();
        assert_eq!(
            Vttc::decode(&mut buf.as_slice()).unwrap(),
            Vttc {
                vsid: Some(0x01020304),
                payl: "Hi".into(),
                ..Default::default()
            }
        );

        // A source ID that isn't 4 bytes is rejected.
        let mut buf = Vec::new();
        buf.encode_atom(Vttc::KIND, |buf| {
            buf.encode_atom_string(VSID, "abcde")?;
            buf.encode_atom_string(PAYL, "Hi")
        })
        .unwrap();
        assert!(Vttc::decode(&mut buf.as_slice()).is_err());
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
