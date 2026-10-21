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
/// before using the values. Except for the first-sample-flags layout, the
/// encoder rejects fields that are present for only some entries because a
/// trun box cannot represent them without changing their meaning.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrunEntry {
    pub duration: Option<u32>,
    pub size: Option<u32>,
    pub flags: Option<u32>,
    /// Composition time offset. Version 0 stores an unsigned `u32`, while
    /// version 1 stores a signed `i32`, so an `i64` is needed to represent
    /// every value from either version.
    pub cts: Option<i64>,
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

        if ext.first_sample_flags && ext.sample_flags {
            return Err(Error::Unsupported(
                "trun first_sample_flags and sample_flags cannot both be set",
            ));
        }

        if ext.first_sample_flags && sample_count == 0 {
            return Err(Error::Unsupported(
                "trun first_sample_flags requires at least one sample",
            ));
        }

        let mut first_sample_flags = match ext.first_sample_flags {
            true => u32::decode(buf)?.into(),
            false => None,
        };

        // Avoid a memory exhaustion attack.
        // If none of the flags are set, then the trun entry has zero size, then we'll allocate `sample_count` entries.
        // Rather than make the API worse, we just limit the number of (useless?) identical entries to 4096.
        if !(ext.sample_duration
            || ext.sample_size
            || ext.sample_flags
            || ext.sample_cts
            || sample_count <= 4096)
        {
            return Err(Error::OutOfMemory);
        }

        let mut entries = Vec::with_capacity(sample_count.min(4096) as _);

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
                true => Some(match ext.version {
                    TrunVersion::V0 => i64::from(u32::decode(buf)?),
                    TrunVersion::V1 => i64::from(i32::decode(buf)?),
                }),
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
        fn field_is_uniform<T>(
            entries: &[TrunEntry],
            get: impl Fn(&TrunEntry) -> Option<T>,
        ) -> bool {
            let Some(first) = entries.first() else {
                return true;
            };
            let present = get(first).is_some();
            entries
                .iter()
                .skip(1)
                .all(|entry| get(entry).is_some() == present)
        }

        if !field_is_uniform(&self.entries, |entry| entry.duration) {
            return Err(Error::Unsupported("mixed trun sample_duration presence"));
        }
        if !field_is_uniform(&self.entries, |entry| entry.size) {
            return Err(Error::Unsupported("mixed trun sample_size presence"));
        }
        if !field_is_uniform(&self.entries, |entry| entry.cts) {
            return Err(Error::Unsupported("mixed trun sample_cts presence"));
        }

        let all_flags =
            !self.entries.is_empty() && self.entries.iter().all(|entry| entry.flags.is_some());
        let first_only_flags = self
            .entries
            .first()
            .is_some_and(|entry| entry.flags.is_some())
            && self.entries.iter().skip(1).all(|s| s.flags.is_none());

        if self.entries.iter().any(|entry| entry.flags.is_some()) && !all_flags && !first_only_flags
        {
            return Err(Error::Unsupported("mixed trun sample_flags presence"));
        }

        let sample_cts = self
            .entries
            .first()
            .is_some_and(|entry| entry.cts.is_some());
        let has_negative_cts = self
            .entries
            .iter()
            .filter_map(|entry| entry.cts)
            .any(|cts| cts < 0);
        let has_large_cts = self
            .entries
            .iter()
            .filter_map(|entry| entry.cts)
            .any(|cts| cts > i64::from(i32::MAX));

        if self
            .entries
            .iter()
            .filter_map(|entry| entry.cts)
            .any(|cts| cts < i64::from(i32::MIN) || cts > i64::from(u32::MAX))
        {
            return Err(Error::Unsupported("trun sample_cts is out of range"));
        }
        if has_negative_cts && has_large_cts {
            return Err(Error::Unsupported(
                "trun sample_cts values require incompatible versions",
            ));
        }

        let version = if has_large_cts {
            TrunVersion::V0
        } else {
            TrunVersion::V1
        };

        let ext = TrunExt {
            version,
            data_offset: self.data_offset.is_some(),
            first_sample_flags: first_only_flags,
            sample_duration: self
                .entries
                .first()
                .is_some_and(|entry| entry.duration.is_some()),
            sample_size: self
                .entries
                .first()
                .is_some_and(|entry| entry.size.is_some()),
            sample_flags: all_flags && !first_only_flags,
            sample_cts,
        };

        (self.entries.len() as u32).encode(buf)?;

        self.data_offset.encode(buf)?;
        if ext.first_sample_flags {
            self.entries[0].flags.unwrap().encode(buf)?;
        }

        for entry in &self.entries {
            if ext.sample_duration {
                entry.duration.unwrap().encode(buf)?;
            }
            if ext.sample_size {
                entry.size.unwrap().encode(buf)?;
            }
            if ext.sample_flags {
                entry.flags.unwrap().encode(buf)?;
            }
            if ext.sample_cts {
                match ext.version {
                    TrunVersion::V0 => (entry.cts.unwrap() as u32).encode(buf)?,
                    TrunVersion::V1 => (entry.cts.unwrap() as i32).encode(buf)?,
                }
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

    /// A mixed per-sample layout cannot be encoded without changing `None`
    /// (inherit the tfhd default) into an explicit value.
    #[test]
    fn mixed_flags_are_rejected() {
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
        assert!(matches!(
            trun.encode(&mut buf),
            Err(Error::Unsupported("mixed trun sample_flags presence"))
        ));
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

    /// A partially present duration field cannot be represented by trun flags.
    #[test]
    fn mixed_duration_is_rejected() {
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
        assert!(matches!(
            trun.encode(&mut buf),
            Err(Error::Unsupported("mixed trun sample_duration presence"))
        ));
    }

    /// A partially present size field cannot be represented by trun flags.
    #[test]
    fn mixed_size_is_rejected() {
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
        assert!(matches!(
            trun.encode(&mut buf),
            Err(Error::Unsupported("mixed trun sample_size presence"))
        ));
    }

    #[test]
    fn mixed_cts_is_rejected() {
        let trun = Trun {
            data_offset: None,
            entries: vec![
                TrunEntry {
                    cts: Some(0),
                    ..Default::default()
                },
                TrunEntry::default(),
            ],
        };

        let mut encoded = Vec::new();
        assert!(matches!(
            trun.encode(&mut encoded),
            Err(Error::Unsupported("mixed trun sample_cts presence"))
        ));
    }

    #[test]
    fn version_zero_cts_is_unsigned() {
        let encoded = [
            0, 0, 0, 20, b't', b'r', b'u', b'n', // size and kind
            0, 0, 8, 0, // version 0, sample_cts flag
            0, 0, 0, 1, // sample_count
            0x80, 0, 0, 0, // unsigned composition offset
        ];

        let trun = Trun::decode(&mut &encoded[..]).expect("decode");
        assert_eq!(trun.entries[0].cts, Some(0x8000_0000));

        let mut reencoded = Vec::new();
        trun.encode(&mut reencoded).expect("re-encode");
        assert_eq!(reencoded[8], 0, "large unsigned offsets require version 0");
        assert_eq!(
            Trun::decode(&mut &reencoded[..]).expect("decode re-encoded"),
            trun
        );
    }

    #[test]
    fn version_one_cts_is_signed() {
        let trun = Trun {
            data_offset: None,
            entries: vec![TrunEntry {
                cts: Some(-1),
                ..Default::default()
            }],
        };

        let mut encoded = Vec::new();
        trun.encode(&mut encoded).expect("encode");
        assert_eq!(encoded[8], 1, "negative offsets require version 1");
        assert_eq!(Trun::decode(&mut &encoded[..]).expect("decode"), trun);
    }

    #[test]
    fn incompatible_cts_versions_are_rejected() {
        let trun = Trun {
            data_offset: None,
            entries: vec![
                TrunEntry {
                    cts: Some(-1),
                    ..Default::default()
                },
                TrunEntry {
                    cts: Some(i64::from(i32::MAX) + 1),
                    ..Default::default()
                },
            ],
        };

        let mut encoded = Vec::new();
        assert!(matches!(
            trun.encode(&mut encoded),
            Err(Error::Unsupported(
                "trun sample_cts values require incompatible versions"
            ))
        ));
    }

    #[test]
    fn first_and_per_sample_flags_are_rejected() {
        let encoded = [
            0, 0, 0, 24, b't', b'r', b'u', b'n', // size and kind
            0, 0, 4, 4, // version 0, both flag fields
            0, 0, 0, 1, // sample_count
            0, 0, 0, 1, // first_sample_flags
            0, 0, 0, 2, // sample_flags for sample 0
        ];

        assert!(matches!(
            Trun::decode(&mut &encoded[..]),
            Err(Error::Unsupported(
                "trun first_sample_flags and sample_flags cannot both be set"
            ))
        ));
    }

    /// When all entries have None for a field, the flag is not set and
    /// the field is omitted entirely (no unnecessary bytes written).
    #[test]
    fn all_none_fields_omitted() {
        let trun = Trun {
            data_offset: Some(100),
            entries: vec![
                TrunEntry {
                    duration: None,
                    size: None,
                    flags: None,
                    cts: None,
                },
                TrunEntry {
                    duration: None,
                    size: None,
                    flags: None,
                    cts: None,
                },
            ],
        };

        let mut buf = Vec::new();
        trun.encode(&mut buf).expect("encode");

        let decoded = Trun::decode(&mut &buf[..]).expect("decode");

        assert_eq!(decoded.entries[0].duration, None);
        assert_eq!(decoded.entries[0].size, None);
        assert_eq!(decoded.entries[0].flags, None);
        assert_eq!(decoded.entries[0].cts, None);
    }
}
