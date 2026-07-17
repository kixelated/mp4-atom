use crate::*;

ext! {
    name: Elst,
    versions: [0, 1],
    flags: {}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Elst {
    pub entries: Vec<ElstEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ElstEntry {
    pub segment_duration: u64,
    /// Start time within the media of this edit, in media-timescale units
    /// (composition time). Signed on the wire (ISO/IEC 14496-12 §8.6.6): `None`
    /// is the `-1` "empty edit" sentinel -- a dwell with no media, used to signal
    /// an initial presentation offset -- and `Some(t)` is a real, non-negative
    /// media time. `t` must fit in an `i64` (the on-wire field is signed) to encode.
    pub media_time: Option<u64>,
    // Signed 16.16 fixed-point playback rate (int(16).int(16) per §8.6.6). Always
    // present -- there is no "absent" sentinel; media_rate == 0 is a meaningful value
    // (a dwell, i.e. a frozen frame) rather than absence -- so it is not optional.
    pub media_rate: FixedPoint<i16>,
}

impl AtomExt for Elst {
    type Ext = ElstExt;

    const KIND_EXT: FourCC = FourCC::new(b"elst");

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: ElstExt) -> Result<Self> {
        let entry_count = u32::decode(buf)?;

        let mut entries = Vec::new();
        for _ in 0..entry_count {
            // media_time is signed; decode it as i32/i64 so the -1 empty-edit
            // sentinel sign-extends instead of becoming +4294967295.
            let (segment_duration, media_time) = match ext.version {
                ElstVersion::V1 => (u64::decode(buf)?, i64::decode(buf)?),
                ElstVersion::V0 => (u32::decode(buf)? as u64, i32::decode(buf)? as i64),
            };

            // -1 is the only defined negative (empty edit); real media times are
            // non-negative. Anything below -1 is out of spec.
            let media_time = match media_time {
                -1 => None,
                t if t >= 0 => Some(t as u64),
                _ => {
                    return Err(Error::Unsupported(
                        "elst media_time must be -1 or non-negative",
                    ))
                }
            };

            entries.push(ElstEntry {
                segment_duration,
                media_time,
                media_rate: FixedPoint::decode(buf)?,
            });
        }

        Ok(Elst { entries })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<ElstExt> {
        // On the wire media_time is signed: None is the -1 empty edit, and a real
        // media time must fit the signed 64-bit field.
        let media_times = self
            .entries
            .iter()
            .map(|e| match e.media_time {
                None => Ok(-1i64),
                Some(t) => i64::try_from(t)
                    .map_err(|_| Error::Unsupported("elst media_time exceeds i64::MAX")),
            })
            .collect::<Result<Vec<_>>>()?;

        // Prefer version 0 (32-bit) when every value fits: it matches what muxers
        // typically emit (so a V0 source round-trips byte-for-byte) and keeps the box
        // compact. Fall back to version 1 (64-bit) when a value is too large.
        let use_v0 =
            self.entries.iter().zip(&media_times).all(|(e, &mt)| {
                u32::try_from(e.segment_duration).is_ok() && i32::try_from(mt).is_ok()
            });

        (self.entries.len() as u32).encode(buf)?;

        for (entry, &media_time) in self.entries.iter().zip(&media_times) {
            if use_v0 {
                (entry.segment_duration as u32).encode(buf)?;
                (media_time as i32).encode(buf)?;
            } else {
                entry.segment_duration.encode(buf)?;
                media_time.encode(buf)?;
            }
            entry.media_rate.encode(buf)?;
        }

        Ok(if use_v0 {
            ElstVersion::V0.into()
        } else {
            ElstVersion::V1.into()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elst32() {
        let expected = Elst {
            entries: vec![ElstEntry {
                segment_duration: 634634,
                media_time: Some(0),
                media_rate: 1.into(),
            }],
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();
        assert_eq!(buf[8], 0, "values within 32 bits encode as version 0");

        let mut buf = buf.as_ref();
        let decoded = Elst::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_elst64() {
        let expected = Elst {
            entries: vec![ElstEntry {
                segment_duration: 5_000_000_000,
                media_time: Some(5_000_000_000),
                media_rate: 1.into(),
            }],
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();
        assert_eq!(buf[8], 1, "values beyond 32 bits force version 1");

        let mut buf = buf.as_ref();
        let decoded = Elst::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    // Regression: the "empty edit" (media_time = -1) must round-trip as `None`.
    // Decoding the 32-bit form as unsigned (the old bug) turned -1 into +4294967295,
    // which shifted the track by 2^32 media ticks and left video/audio on disjoint
    // timelines -- a black screen in the browser's Media Source Extensions.
    #[test]
    fn test_elst_empty_edit_sentinel() {
        let expected = Elst {
            entries: vec![
                ElstEntry {
                    segment_duration: 23,
                    media_time: None,
                    media_rate: 1.into(),
                },
                ElstEntry {
                    segment_duration: 0,
                    media_time: Some(0),
                    media_rate: 1.into(),
                },
            ],
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();
        let decoded = Elst::decode(&mut buf.as_ref()).unwrap();
        assert_eq!(decoded, expected);
    }

    // Decode a real version-0 edit list (as written by ffmpeg / L-SMASH etc.) whose
    // media_time is the 32-bit -1 (0xFFFFFFFF), and confirm we recover `None` and
    // preserve it through a re-encode (the decode -> re-encode path a repackager takes).
    #[test]
    fn test_elst_v0_neg1_decode_reencode() {
        let raw: &[u8] = &[
            0x00, 0x00, 0x00, 0x1C, // box size = 28
            b'e', b'l', b's', b't', //
            0x00, 0x00, 0x00, 0x00, // version 0, flags 0
            0x00, 0x00, 0x00, 0x01, // entry_count = 1
            0x00, 0x00, 0x03, 0xE8, // segment_duration = 1000
            0xFF, 0xFF, 0xFF, 0xFF, // media_time = -1 (32-bit)
            0x00, 0x01, 0x00, 0x00, // media_rate = 1.0
        ];
        let decoded = Elst::decode(&mut &raw[..]).unwrap();
        assert_eq!(decoded.entries[0].media_time, None);

        // A version-0 input round-trips byte-for-byte: the empty edit stays a 32-bit
        // -1, not a re-widened (and previously corrupted) 64-bit value.
        let mut buf = Vec::new();
        decoded.encode(&mut buf).unwrap();
        assert_eq!(buf.as_slice(), raw);
    }

    // A media_time below -1 is out of spec: decoding must fail rather than invent a value.
    #[test]
    fn test_elst_negative_media_time_rejected() {
        let raw: &[u8] = &[
            0x00, 0x00, 0x00, 0x1C, // box size = 28
            b'e', b'l', b's', b't', //
            0x00, 0x00, 0x00, 0x00, // version 0, flags 0
            0x00, 0x00, 0x00, 0x01, // entry_count = 1
            0x00, 0x00, 0x03, 0xE8, // segment_duration = 1000
            0xFF, 0xFF, 0xFF, 0xFE, // media_time = -2 (invalid)
            0x00, 0x01, 0x00, 0x00, // media_rate = 1.0
        ];
        assert!(Elst::decode(&mut &raw[..]).is_err());
    }

    // A media_time that does not fit the signed on-wire field cannot be encoded.
    #[test]
    fn test_elst_media_time_too_large_rejected() {
        let elst = Elst {
            entries: vec![ElstEntry {
                segment_duration: 0,
                media_time: Some(u64::MAX),
                media_rate: 1.into(),
            }],
        };
        let mut buf = Vec::new();
        assert!(elst.encode(&mut buf).is_err());
    }
}
