use crate::*;

/// `EventMessageInstanceBox` ('emib'), the sample format used by CMAF/DASH
/// Event Message tracks (the [`Evte`](crate::Evte) sample entry). Carries the
/// same information as the top-level [`Emsg`] box, but as sample content
/// (i.e. inside `mdat`) rather than a structural box, so it must be decoded
/// explicitly from the sample bytes rather than via [`Any`].
///
/// See ISO/IEC 23001-18:2022 Section 6.1.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Emib {
    /// The event start time on the media presentation timeline, relative to
    /// the presentation time of the sample enclosing this box, in units of
    /// the track's `Mdhd` timescale.
    pub presentation_time_delta: i64,
    pub event_duration: u32,
    pub id: u32,
    pub scheme_id_uri: String,
    pub value: String,
    pub message_data: Vec<u8>,
}

impl AtomExt for Emib {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"emib");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        u32::decode(buf)?; // reserved
        let presentation_time_delta = i64::decode(buf)?;
        let event_duration = u32::decode(buf)?;
        let id = u32::decode(buf)?;
        let scheme_id_uri = String::decode(buf)?;
        let value = String::decode(buf)?;
        let message_data = Vec::decode(buf)?;

        Ok(Self {
            presentation_time_delta,
            event_duration,
            id,
            scheme_id_uri,
            value,
            message_data,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        0u32.encode(buf)?; // reserved
        self.presentation_time_delta.encode(buf)?;
        self.event_duration.encode(buf)?;
        self.id.encode(buf)?;
        self.scheme_id_uri.as_str().encode(buf)?;
        self.value.as_str().encode(buf)?;
        self.message_data.encode(buf)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A real `emib` sample from the MPEG conformance file `out_avail_track.cmfm`.
    const ENCODED_EMIB: &[u8] = &[
        0x00, 0x00, 0x00, 0x5e, 0x65, 0x6d, 0x69, 0x62, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x75, 0x30, 0x00, 0x00,
        0x00, 0x00, 0x75, 0x72, 0x6e, 0x3a, 0x73, 0x63, 0x74, 0x65, 0x3a, 0x73, 0x63, 0x74, 0x65,
        0x33, 0x35, 0x3a, 0x32, 0x30, 0x31, 0x33, 0x3a, 0x62, 0x69, 0x6e, 0x00, 0x00, 0xfc, 0x30,
        0x21, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xf0, 0x10, 0x05, 0x00, 0x00, 0x00,
        0x00, 0x7f, 0xef, 0x7f, 0xfe, 0x00, 0x29, 0x32, 0xe0, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00,
        0xe4, 0x61, 0x24, 0x02,
    ];

    #[test]
    fn test_emib_decode() {
        let emib =
            Emib::decode(&mut std::io::Cursor::new(ENCODED_EMIB)).expect("failed to decode emib");

        assert_eq!(emib.presentation_time_delta, 0);
        assert_eq!(emib.event_duration, 30000);
        assert_eq!(emib.id, 0);
        assert_eq!(emib.scheme_id_uri, "urn:scte:scte35:2013:bin");
        assert_eq!(emib.value, "");
        assert_eq!(emib.message_data.len(), 36);
    }

    #[test]
    fn test_emib_roundtrip() {
        let emib =
            Emib::decode(&mut std::io::Cursor::new(ENCODED_EMIB)).expect("failed to decode emib");

        let mut buf = Vec::new();
        emib.encode(&mut buf).unwrap();
        assert_eq!(buf.as_slice(), ENCODED_EMIB);
    }
}
