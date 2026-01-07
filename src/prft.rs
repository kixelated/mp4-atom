use crate::*;

// ProducerReferenceTimeBox, ISO/IEC 14496-12 Section 8.16.5
// This is called out in CMAF (23000-19) and DASH (23009-1), optional.

ext! {
    name: Prft,
    versions: [0, 1],
    flags: {
        output_time = 0,
        fragment_finalised = 1,
        fragment_written = 2,
        consistent_offset = 3,
        real_time = 4,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ReferenceTime {
    /// The UTC time is the time at which the frame belonging to
    /// the reference track in the following movie fragment and
    /// whose presentation time is `media_time` was input to the encoder.
    #[default]
    Input,

    /// The UTC time is the time at which the frame belonging to
    /// the reference track in the following movie fragment and
    /// whose presentation time is `media_time` was output from the encoder.
    Output,

    /// The UTC time is the time at which the following `MovieFragmentBox`
    /// was finalized. `media_time` is set to the presentation of
    /// the earliest frame of the reference track in presentation order
    /// of the movie fragment.
    Finalised,

    /// The UTC time is the time at which the following `MovieFragmentBox`
    /// was written to file. `media_time` is set to the presentation of
    /// the earliest frame of the reference track in presentation order
    /// of the movie fragment.
    Written,

    /// The association between the `media_time` and UTC time is arbitrary
    /// but consistent between  multiple occurrences of this box in the same track.
    Consistent,

    /// The UTC time has a consistent, small (ideally zero), offset from the
    /// real-time of the experience depicted in the media at `media_time`.
    RealTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Prft {
    pub reference_track_id: u32,
    pub ntp_timestamp: u64,
    pub media_time: u64,
    pub utc_time_semantics: ReferenceTime,
}

impl AtomExt for Prft {
    type Ext = PrftExt;

    const KIND_EXT: FourCC = FourCC::new(b"prft");

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: PrftExt) -> Result<Self> {
        let reference_track_id = u32::decode(buf)?;
        let ntp_timestamp = u64::decode(buf)?;
        let utc_time_semantics = if ext.real_time && ext.consistent_offset {
            ReferenceTime::RealTime
        } else if ext.consistent_offset {
            ReferenceTime::Consistent
        } else if ext.fragment_written {
            ReferenceTime::Written
        } else if ext.fragment_finalised {
            ReferenceTime::Finalised
        } else if ext.output_time {
            ReferenceTime::Output
        } else {
            // fallback
            ReferenceTime::Input
        };
        if ext.version == PrftVersion::V0 {
            Ok(Prft {
                reference_track_id,
                ntp_timestamp,
                media_time: u32::decode(buf)?.into(),
                utc_time_semantics,
            })
        } else {
            Ok(Prft {
                reference_track_id,
                ntp_timestamp,
                media_time: u64::decode(buf)?,
                utc_time_semantics,
            })
        }
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<PrftExt> {
        self.reference_track_id.encode(buf)?;
        self.ntp_timestamp.encode(buf)?;
        let (output_time, fragment_finalised, fragment_written, consistent_offset, real_time) =
            match self.utc_time_semantics {
                ReferenceTime::Input => (false, false, false, false, false),
                ReferenceTime::Output => (true, false, false, false, false),
                ReferenceTime::Finalised => (false, true, false, false, false),
                ReferenceTime::Written => (false, false, true, false, false),
                ReferenceTime::Consistent => (false, false, false, true, false),
                ReferenceTime::RealTime => (false, false, false, true, true),
            };
        if self.media_time <= u32::MAX.into() {
            (self.media_time as u32).encode(buf)?;
            Ok(PrftExt {
                version: PrftVersion::V0,
                output_time,
                fragment_finalised,
                fragment_written,
                consistent_offset,
                real_time,
            })
        } else {
            self.media_time.encode(buf)?;
            Ok(PrftExt {
                version: PrftVersion::V1,
                output_time,
                fragment_finalised,
                fragment_written,
                consistent_offset,
                real_time,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // From MPEG File Format Conformance suite: 21_segment.mp4
    const ENCODED_PRFT: &[u8] = &[
        0x00, 0x00, 0x00, 0x20, 0x70, 0x72, 0x66, 0x74, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0xda, 0x74, 0xca, 0x46, 0x6b, 0xc6, 0xa7, 0xef, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xf8,
    ];

    // Decoded values per 21_segment_gpac.json
    const DECODED_PRFT: Prft = Prft {
        reference_track_id: 1,
        ntp_timestamp: 15741429001371428847,
        media_time: 18446744073709551608,
        utc_time_semantics: ReferenceTime::Input,
    };

    #[test]
    fn test_prft_v1_decode() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_PRFT);
        let prft = Prft::decode(buf).expect("failed to decode prft");
        assert_eq!(prft, DECODED_PRFT);
    }

    #[test]
    fn test_prft_v1_encode() {
        let mut buf = Vec::new();
        DECODED_PRFT.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_PRFT);
    }

    #[test]
    fn test_prft_v0_round_trip() {
        let mut buf = Vec::new();
        let prft = Prft {
            reference_track_id: 7,
            ntp_timestamp: 15741429001371428847,
            media_time: u32::MAX.into(),
            utc_time_semantics: ReferenceTime::Written,
        };
        prft.encode(&mut buf).unwrap();
        assert_eq!(
            buf.as_slice(),
            &[
                0x00, 0x00, 0x00, 0x1C, 0x70, 0x72, 0x66, 0x74, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00,
                0x00, 0x07, 0xda, 0x74, 0xca, 0x46, 0x6b, 0xc6, 0xa7, 0xef, 0xff, 0xff, 0xff, 0xff
            ]
        );

        let decoded = Prft::decode(&mut buf.as_ref()).unwrap();
        assert_eq!(decoded, prft);
    }

    #[test]
    fn test_prft_realtime_roundtrip() {
        let mut buf = Vec::new();
        let prft = Prft {
            reference_track_id: 1,
            ntp_timestamp: 16571585696146385000,
            media_time: 41234604048,
            utc_time_semantics: ReferenceTime::RealTime,
        };
        prft.encode(&mut buf).unwrap();
        assert_eq!(
            buf.as_slice(),
            &[
                0x00, 0x00, 0x00, 0x20, 0x70, 0x72, 0x66, 0x74, 0x01, 0x00, 0x00, 0x18, 0x00, 0x00,
                0x00, 0x01, 0xe5, 0xfa, 0x19, 0x63, 0xff, 0xbf, 0xe8, 0x68, 0x00, 0x00, 0x00, 0x09,
                0x99, 0xc6, 0x20, 0x10
            ]
        );

        let decoded = Prft::decode(&mut buf.as_ref()).unwrap();
        assert_eq!(decoded, prft);
    }
}
