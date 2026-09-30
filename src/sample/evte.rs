use std::borrow::Borrow;

use crate::*;

/// The content of a single sample in an Event Message track (the
/// [`Evte`](crate::Evte) sample entry).
///
/// Sample content lives inside `mdat` and isn't walked automatically by this
/// crate (it isn't part of the structural box tree, and [`Emib`]/[`Emeb`]
/// aren't registered in [`Any`] since they only ever appear here, as sample
/// data) -- decode it explicitly via [`EvteSample::decode`].
///
/// See ISO/IEC 23001-18:2022 Section 6.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EvteSample;

impl EvteSample {
    pub fn decode<B: Buf>(buf: &mut B) -> Result<Vec<Emib>> {
        let mut emibs = Vec::new();
        let mut emeb = None;

        while let Some(header) = Header::decode_maybe(buf)? {
            let size = header.size.unwrap_or(buf.remaining());
            if size > buf.remaining() {
                return Err(Error::OutOfBounds);
            }
            let mut body = buf.slice(size);

            match header.kind {
                Emib::KIND if emeb.is_none() => emibs.push(Emib::decode_body(&mut body)?),
                Emeb::KIND if emeb.is_none() && emibs.is_empty() => {
                    emeb = Some(Emeb::decode_body(&mut body)?)
                }
                kind => return Err(Error::UnexpectedBox(kind)),
            }

            if body.has_remaining() {
                return Err(Error::UnderDecode(header.kind));
            }
            buf.advance(size);
        }

        if buf.has_remaining() {
            Err(Error::ShortRead)
        } else if emeb.is_none() && emibs.is_empty() {
            Err(Error::UnexpectedEof)
        } else {
            Ok(emibs)
        }
    }

    pub fn encode<B: BufMut>(
        emibs: impl IntoIterator<Item = impl Borrow<Emib>>,
        buf: &mut B,
    ) -> Result<()> {
        let mut wrote_atom = false;
        for emib in emibs {
            emib.borrow().encode(buf)?;
            wrote_atom = true;
        }
        if wrote_atom {
            Ok(())
        } else {
            Emeb.encode(buf)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_emib(id: u32) -> Emib {
        Emib {
            presentation_time_delta: 0,
            event_duration: 1000,
            id,
            scheme_id_uri: "urn:mpeg:dash:event:2012".into(),
            value: "".into(),
            message_data: vec![1, 2, 3],
        }
    }

    #[test]
    fn test_evte_sample_no_emib_roundtrip() {
        let sample = vec![];

        let mut buf = Vec::new();
        EvteSample::encode(&sample, &mut buf).unwrap();

        let decoded = EvteSample::decode(&mut buf.as_slice()).expect("failed to decode sample");
        assert_eq!(decoded, sample);
    }

    #[test]
    fn test_evte_sample_single_emib_roundtrip() {
        let sample = vec![sample_emib(1)];

        let mut buf = Vec::new();
        EvteSample::encode(&sample, &mut buf).unwrap();

        let decoded = EvteSample::decode(&mut buf.as_slice()).expect("failed to decode sample");
        assert_eq!(decoded, sample);
    }

    #[test]
    fn test_evte_sample_multiple_emib_roundtrip() {
        // A sample may concatenate multiple concurrent event instances.
        let sample = vec![sample_emib(1), sample_emib(2), sample_emib(3)];

        let mut buf = Vec::new();
        EvteSample::encode(&sample, &mut buf).unwrap();

        let decoded = EvteSample::decode(&mut buf.as_slice()).expect("failed to decode sample");
        assert_eq!(decoded, sample);
    }

    #[test]
    fn test_evte_sample_empty_is_error() {
        let err = EvteSample::decode(&mut [].as_slice()).unwrap_err();
        assert!(matches!(err, Error::UnexpectedEof));
    }

    #[test]
    fn test_evte_sample_short_read() {
        let err = EvteSample::decode(&mut [1, 2].as_slice()).unwrap_err();
        assert!(matches!(err, Error::ShortRead));
    }

    #[test]
    fn test_evte_sample_emeb_after_emib_is_error() {
        // `emeb` means "no event"; it must not be mixed with `emib`s.
        let mut buf = Vec::new();
        sample_emib(1).encode(&mut buf).unwrap();
        Emeb.encode(&mut buf).unwrap();

        let err = EvteSample::decode(&mut buf.as_slice()).unwrap_err();
        assert!(matches!(err, Error::UnexpectedBox(kind) if kind == Emeb::KIND));
    }

    #[test]
    fn test_evte_sample_invalid_emeb_is_error() {
        let mut buf = Vec::new();

        let header = Header {
            kind: Emeb::KIND,
            size: Some(size_of::<u32>()),
        };
        header.encode(&mut buf).unwrap();
        0u32.encode(&mut buf).unwrap(); // invalid `emeb` body

        let err = EvteSample::decode(&mut buf.as_slice()).unwrap_err();
        assert!(matches!(err, Error::UnderDecode(kind) if kind == Emeb::KIND));
    }
}
