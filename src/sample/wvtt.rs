use std::borrow::Borrow;

use crate::*;

/// One box of a [`WvttSample`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WvttSampleBox {
    /// `VTTCueBox` ('vttc'): a cue active during the sample.
    Vttc(Vttc),
    /// `VTTAdditionalTextBox` ('vtta'): text between cues, e.g. a comment.
    Vtta(Vtta),
}

/// The content of a single sample in a WebVTT track (the [`Wvtt`] sample
/// entry): either a single `VTTEmptyCueBox` ([`Vtte`], no cue is active), or a
/// concatenation of cue boxes ([`Vttc`]) and additional text boxes
/// ([`Vtta`]).
///
/// Sample content lives inside `mdat` and isn't walked automatically by this
/// crate -- decode it explicitly via [`WvttSample::decode`]. The boxes are
/// kept in file order, so that a sample encodes back to the bytes it was
/// decoded from. An empty list of boxes is the `vtte` sample.
///
/// See ISO/IEC 14496-30:2018 Section 6.6.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WvttSample;

impl WvttSample {
    pub fn decode<B: Buf>(buf: &mut B) -> Result<Vec<WvttSampleBox>> {
        let mut boxes = Vec::new();
        let mut empty = false;

        while let Some(header) = Header::decode_maybe(buf)? {
            match header.kind {
                Vttc::KIND if !empty => {
                    boxes.push(WvttSampleBox::Vttc(Vttc::decode_atom(&header, buf)?))
                }
                Vtta::KIND if !empty => {
                    boxes.push(WvttSampleBox::Vtta(Vtta::decode_atom(&header, buf)?))
                }
                Vtte::KIND if !empty && boxes.is_empty() => {
                    Vtte::decode_atom(&header, buf)?;
                    empty = true;
                }
                kind => return Err(Error::UnexpectedBox(kind)),
            }
        }

        if buf.has_remaining() {
            Err(Error::ShortRead)
        } else if !empty && boxes.is_empty() {
            Err(Error::UnexpectedEof)
        } else {
            Ok(boxes)
        }
    }

    pub fn encode<B: BufMut>(
        boxes: impl IntoIterator<Item = impl Borrow<WvttSampleBox>>,
        buf: &mut B,
    ) -> Result<()> {
        let mut wrote_atom = false;
        for item in boxes {
            match item.borrow() {
                WvttSampleBox::Vttc(vttc) => vttc.encode(buf)?,
                WvttSampleBox::Vtta(vtta) => vtta.encode(buf)?,
            }
            wrote_atom = true;
        }
        if wrote_atom {
            Ok(())
        } else {
            Vtte.encode(buf)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(sample: &[WvttSampleBox]) {
        let mut buf = Vec::new();
        WvttSample::encode(sample, &mut buf).unwrap();
        let decoded = WvttSample::decode(&mut buf.as_slice()).expect("failed to decode sample");
        assert_eq!(decoded, sample);
    }

    #[test]
    fn test_wvtt_sample_cues() {
        roundtrip(&[
            WvttSampleBox::Vtta(Vtta {
                text: "NOTE hi".into(),
            }),
            WvttSampleBox::Vttc(Vttc {
                vsid: None,
                iden: Some("1".into()),
                ctim: None,
                sttg: Some("align:start".into()),
                payl: "Hello".into(),
            }),
            WvttSampleBox::Vtta(Vtta {
                text: "NOTE again".into(),
            }),
            WvttSampleBox::Vttc(Vttc {
                payl: "World".into(),
                ..Default::default()
            }),
        ]);
    }

    #[test]
    fn test_wvtt_sample_empty_cue() {
        let mut buf = Vec::new();
        WvttSample::encode(&[] as &[WvttSampleBox], &mut buf).unwrap();
        assert_eq!(buf, [0, 0, 0, 8, b'v', b't', b't', b'e']);
        roundtrip(&[]);
    }

    #[test]
    fn test_wvtt_sample_empty_with_other_boxes_is_error() {
        let mut buf = Vec::new();
        Vtte.encode(&mut buf).unwrap();
        Vtta::default().encode(&mut buf).unwrap();
        let err = WvttSample::decode(&mut buf.as_slice()).unwrap_err();
        assert!(matches!(err, Error::UnexpectedBox(kind) if kind == Vtta::KIND));

        let mut buf = Vec::new();
        Vtta::default().encode(&mut buf).unwrap();
        Vtte.encode(&mut buf).unwrap();
        let err = WvttSample::decode(&mut buf.as_slice()).unwrap_err();
        assert!(matches!(err, Error::UnexpectedBox(kind) if kind == Vtte::KIND));
    }

    #[test]
    fn test_wvtt_sample_empty_is_error() {
        let err = WvttSample::decode(&mut [].as_slice()).unwrap_err();
        assert!(matches!(err, Error::UnexpectedEof));
    }
}
