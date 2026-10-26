use crate::*;

ext! {
    name: Trun,
    versions: [0, 1],
    flags: {
        data_offset = 0,
        first_sample_flags = 2,
        sample_duration = 8,
        sample_size = 9,
        sample_flags = 10,
        sample_cts = 11,
    }
}

/// A sample run with at least one per-sample field when nonempty.
///
/// Runs containing only a sample count are unsupported: this representation
/// materializes each entry, so every entry needs bytes in the input to bound it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Trun {
    pub data_offset: Option<i32>,
    pub entries: Vec<TrunEntry>,
}

/// A single sample entry in a trun box.
///
/// `None` fields mean the value was not present in the per-sample trun data.
/// After decode, callers should resolve `None` against tfhd defaults
/// (`default_sample_duration`, `default_sample_size`, `default_sample_flags`)
/// before using the values. The encoder backfills unresolved `None` with `0`
/// as a last resort to avoid silently dropping fields that other entries set.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrunEntry {
    pub duration: Option<u32>,
    pub size: Option<u32>,
    pub flags: Option<u32>,
    pub cts: Option<i32>,
}

impl AtomExt for Trun {
    const KIND_EXT: FourCC = FourCC::new(b"trun");

    type Ext = TrunExt;

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: TrunExt) -> Result<Self> {
        let sample_count = u32::decode(buf)?;
        let data_offset = match ext.data_offset {
            true => i32::decode(buf)?.into(),
            false => None,
        };

        let mut first_sample_flags = match ext.first_sample_flags {
            true => u32::decode(buf)?.into(),
            false => None,
        };

        // Every materialized entry needs a per-entry field in the input. A count
        // alone could otherwise expand a small moof into arbitrarily many entries.
        if sample_count != 0
            && !(ext.sample_duration || ext.sample_size || ext.sample_flags || ext.sample_cts)
        {
            return Err(Error::Unsupported(
                "nonempty trun without per-sample fields",
            ));
        }

        // Grow only as fields are read, so a truncated table cannot allocate by count.
        let mut entries = Vec::new();

        for _ in 0..sample_count {
            let duration = match ext.sample_duration {
                true => u32::decode(buf)?.into(),
                false => None,
            };
            let size = match ext.sample_size {
                true => u32::decode(buf)?.into(),
                false => None,
            };
            let sample_flags = match first_sample_flags.take() {
                Some(flags) => Some(flags),
                None => match ext.sample_flags {
                    true => u32::decode(buf)?.into(),
                    false => None,
                },
            };
            let cts = match ext.sample_cts {
                true => i32::decode(buf)?.into(),
                false => None,
            };

            entries.push(TrunEntry {
                duration,
                size,
                flags: sample_flags,
                cts,
            });
        }

        Ok(Trun {
            data_offset,
            entries,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<TrunExt> {
        let any_flags = self.entries.iter().any(|s| s.flags.is_some());
        let first_only_flags = any_flags
            && self.entries.first().is_some_and(|s| s.flags.is_some())
            && self.entries.iter().skip(1).all(|s| s.flags.is_none());

        // Use per-sample flags when any entry has flags and it's not the first-only pattern.
        // None entries are backfilled with 0 to avoid silently dropping flags.
        let sample_flags = any_flags && !first_only_flags;

        let ext = TrunExt {
            version: TrunVersion::V1,
            data_offset: self.data_offset.is_some(),
            first_sample_flags: first_only_flags,

            // Use the field if any entry has it set. None entries are backfilled
            // with 0 during encoding to avoid silently dropping fields on
            // decode→encode roundtrips (entries that inherited defaults from tfhd
            // have None after decode).
            sample_duration: self.entries.iter().any(|s| s.duration.is_some()),
            sample_size: self.entries.iter().any(|s| s.size.is_some()),
            sample_flags,
            sample_cts: self.entries.iter().any(|s| s.cts.is_some()),
        };

        if !self.entries.is_empty()
            && !(ext.sample_duration || ext.sample_size || ext.sample_flags || ext.sample_cts)
        {
            return Err(Error::Unsupported(
                "nonempty trun without per-sample fields",
            ));
        }

        (self.entries.len() as u32).encode(buf)?;

        self.data_offset.encode(buf)?;
        if ext.first_sample_flags {
            self.entries[0].flags.unwrap().encode(buf)?;
        }

        for entry in &self.entries {
            if ext.sample_duration {
                Some(Some(entry.duration.unwrap_or(0))).encode(buf)?;
            }
            if ext.sample_size {
                Some(Some(entry.size.unwrap_or(0))).encode(buf)?;
            }
            if ext.sample_flags {
                Some(Some(entry.flags.unwrap_or(0))).encode(buf)?;
            }
            if ext.sample_cts {
                Some(Some(entry.cts.unwrap_or(0))).encode(buf)?;
            }
        }

        Ok(ext)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Verify that first_sample_flags survives encode→decode roundtrip.
    ///
    /// ffmpeg commonly writes trun boxes where only the first entry has flags
    /// (via first_sample_flags) and the rest inherit default_sample_flags from
    /// tfhd. After decode, entry[0].flags = Some(keyframe), entries[1..N].flags = None.
    /// The encoder must preserve this by emitting first_sample_flags.
    #[test]
    fn first_sample_flags_roundtrip() {
        let trun = Trun {
            data_offset: Some(100),
            entries: vec![
                TrunEntry {
                    duration: Some(512),
                    size: Some(1000),
                    flags: Some(0x02000000), // keyframe (sample_depends_on=2)
                    cts: None,
                },
                TrunEntry {
                    duration: Some(512),
                    size: Some(200),
                    flags: None, // inherits default_sample_flags from tfhd
                    cts: None,
                },
                TrunEntry {
                    duration: Some(512),
                    size: Some(200),
                    flags: None,
                    cts: None,
                },
            ],
        };

        let mut buf = Vec::new();
        trun.encode(&mut buf).expect("encode");

        let decoded = Trun::decode(&mut &buf[..]).expect("decode");

        // entry[0] must have the keyframe flags from first_sample_flags
        assert_eq!(decoded.entries[0].flags, Some(0x02000000));
        // entries[1..N] must have None (they use default_sample_flags from tfhd)
        assert_eq!(decoded.entries[1].flags, None);
        assert_eq!(decoded.entries[2].flags, None);
        assert_eq!(decoded.data_offset, Some(100));
        assert_eq!(decoded.entries.len(), 3);
    }

    /// When multiple entries have explicit flags (not just the first),
    /// the encoder must use per-sample flags, not first_sample_flags.
    #[test]
    fn mixed_flags_uses_per_sample() {
        let trun = Trun {
            data_offset: Some(100),
            entries: vec![
                TrunEntry {
                    duration: Some(512),
                    size: Some(1000),
                    flags: Some(0x02000000), // keyframe
                    cts: None,
                },
                TrunEntry {
                    duration: Some(512),
                    size: Some(200),
                    flags: Some(0x01010000), // non-keyframe (explicit)
                    cts: None,
                },
                TrunEntry {
                    duration: Some(512),
                    size: Some(200),
                    flags: None, // no flags
                    cts: None,
                },
            ],
        };

        let mut buf = Vec::new();
        trun.encode(&mut buf).expect("encode");

        let decoded = Trun::decode(&mut &buf[..]).expect("decode");

        // Mixed Some/None: encoder backfills None with 0 and emits per-sample flags.
        assert_eq!(decoded.entries[0].flags, Some(0x02000000));
        assert_eq!(decoded.entries[1].flags, Some(0x01010000));
        assert_eq!(decoded.entries[2].flags, Some(0)); // was None, backfilled to 0
    }

    /// When all entries have explicit flags, per-sample flags are used.
    #[test]
    fn all_flags_roundtrip() {
        let trun = Trun {
            data_offset: Some(100),
            entries: vec![
                TrunEntry {
                    duration: Some(512),
                    size: Some(1000),
                    flags: Some(0x02000000),
                    cts: None,
                },
                TrunEntry {
                    duration: Some(512),
                    size: Some(200),
                    flags: Some(0x01010000),
                    cts: None,
                },
            ],
        };

        let mut buf = Vec::new();
        trun.encode(&mut buf).expect("encode");

        let decoded = Trun::decode(&mut &buf[..]).expect("decode");

        assert_eq!(decoded.entries[0].flags, Some(0x02000000));
        assert_eq!(decoded.entries[1].flags, Some(0x01010000));
    }

    /// Entries with None duration (inherited from tfhd default_sample_duration)
    /// must not cause the duration field to be dropped entirely on re-encode.
    #[test]
    fn duration_backfill_roundtrip() {
        let trun = Trun {
            data_offset: Some(100),
            entries: vec![
                TrunEntry {
                    duration: Some(512),
                    size: Some(1000),
                    flags: Some(0x02000000),
                    cts: None,
                },
                TrunEntry {
                    duration: None, // inherited from tfhd
                    size: Some(200),
                    flags: None,
                    cts: None,
                },
            ],
        };

        let mut buf = Vec::new();
        trun.encode(&mut buf).expect("encode");

        let decoded = Trun::decode(&mut &buf[..]).expect("decode");

        assert_eq!(decoded.entries[0].duration, Some(512));
        // None backfilled to 0 during encode
        assert_eq!(decoded.entries[1].duration, Some(0));
    }

    /// Entries with None size (inherited from tfhd default_sample_size)
    /// must not cause the size field to be dropped entirely on re-encode.
    #[test]
    fn size_backfill_roundtrip() {
        let trun = Trun {
            data_offset: Some(100),
            entries: vec![
                TrunEntry {
                    duration: Some(512),
                    size: Some(1000),
                    flags: None,
                    cts: None,
                },
                TrunEntry {
                    duration: Some(512),
                    size: None, // inherited from tfhd
                    flags: None,
                    cts: None,
                },
            ],
        };

        let mut buf = Vec::new();
        trun.encode(&mut buf).expect("encode");

        let decoded = Trun::decode(&mut &buf[..]).expect("decode");

        assert_eq!(decoded.entries[0].size, Some(1000));
        // None backfilled to 0 during encode
        assert_eq!(decoded.entries[1].size, Some(0));
    }

    /// Field-less runs have no input bytes to back their sample count.
    #[test]
    fn fieldless_runs_are_refused() {
        for count in [1, 4096, u32::MAX] {
            for first_flags in [false, true] {
                // Full-box header, sample count, optional first flags; no per-entry fields.
                let mut body = vec![0, 0, 0, if first_flags { 4 } else { 0 }];
                body.extend_from_slice(&count.to_be_bytes());
                if first_flags {
                    body.extend_from_slice(&0u32.to_be_bytes());
                }
                assert!(matches!(
                    Trun::decode_body(&mut body.as_slice()),
                    Err(Error::Unsupported(_))
                ));
            }
        }
    }

    #[test]
    fn fieldless_runs_cannot_be_encoded() {
        let run = Trun {
            entries: vec![Default::default(); 4],
            ..Default::default()
        };
        assert!(matches!(
            run.encode(&mut Vec::new()),
            Err(Error::Unsupported(_))
        ));
    }

    #[test]
    fn empty_run_roundtrip() {
        let run = Trun::default();
        let mut buf = Vec::new();
        run.encode(&mut buf).unwrap();
        assert_eq!(Trun::decode(&mut buf.as_slice()).unwrap(), run);
    }
}
