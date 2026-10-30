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
    pub samples: TrunSamples,
}

/// The samples of a trun box.
///
/// A run without any per-sample field is stored as a count, so decoding never
/// allocates in proportion to an untrusted `sample_count`.
/// Use [TrunSamples::iter] to visit every sample's [TrunEntry] in either form.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TrunSamples {
    /// Every sample inherits its duration, size, flags, and composition offset
    /// from tfhd/trex, except that the first sample may override its flags.
    Inherited {
        count: u32,
        first_flags: Option<u32>,
    },

    /// Per-sample entries, used when the run carries at least one per-sample field.
    Explicit(Vec<TrunEntry>),
}

impl TrunSamples {
    /// The number of samples in the run.
    pub fn len(&self) -> usize {
        match self {
            Self::Inherited { count, .. } => *count as usize,
            Self::Explicit(entries) => entries.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The entry of the sample at `index`, or `None` if it is out of range.
    pub fn get(&self, index: usize) -> Option<TrunEntry> {
        match self {
            Self::Inherited { count, first_flags } => {
                (index < *count as usize).then(|| TrunEntry {
                    flags: if index == 0 { *first_flags } else { None },
                    ..Default::default()
                })
            }
            Self::Explicit(entries) => entries.get(index).copied(),
        }
    }

    /// Each sample's entry in order, without allocating.
    pub fn iter(&self) -> TrunSamplesIter<'_> {
        match self {
            Self::Inherited { count, first_flags } => TrunSamplesIter::Inherited {
                remaining: *count,
                first_flags: *first_flags,
            },
            Self::Explicit(entries) => TrunSamplesIter::Explicit(entries.iter()),
        }
    }
}

impl Default for TrunSamples {
    fn default() -> Self {
        Self::Inherited {
            count: 0,
            first_flags: None,
        }
    }
}

impl From<Vec<TrunEntry>> for TrunSamples {
    fn from(entries: Vec<TrunEntry>) -> Self {
        Self::Explicit(entries)
    }
}

impl<'a> IntoIterator for &'a TrunSamples {
    type Item = TrunEntry;
    type IntoIter = TrunSamplesIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over the entries of [TrunSamples].
#[derive(Debug, Clone)]
pub enum TrunSamplesIter<'a> {
    Inherited {
        remaining: u32,
        first_flags: Option<u32>,
    },
    Explicit(std::slice::Iter<'a, TrunEntry>),
}

impl Iterator for TrunSamplesIter<'_> {
    type Item = TrunEntry;

    fn next(&mut self) -> Option<TrunEntry> {
        match self {
            Self::Inherited {
                remaining,
                first_flags,
            } => {
                *remaining = remaining.checked_sub(1)?;
                Some(TrunEntry {
                    flags: first_flags.take(),
                    ..Default::default()
                })
            }
            Self::Explicit(entries) => entries.next().copied(),
        }
    }

    fn nth(&mut self, n: usize) -> Option<TrunEntry> {
        match self {
            Self::Inherited {
                remaining,
                first_flags,
            } => {
                // Every sample after the first is identical, so skip without stepping through them.
                if n > 0 {
                    *first_flags = None;
                    *remaining = remaining.saturating_sub(n.try_into().unwrap_or(u32::MAX));
                }
            }
            Self::Explicit(entries) => return entries.nth(n).copied(),
        }
        self.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = match self {
            Self::Inherited { remaining, .. } => *remaining as usize,
            Self::Explicit(entries) => entries.len(),
        };
        (len, Some(len))
    }
}

impl ExactSizeIterator for TrunSamplesIter<'_> {}

impl Default for TrunSamplesIter<'_> {
    /// An empty iterator.
    fn default() -> Self {
        Self::Inherited {
            remaining: 0,
            first_flags: None,
        }
    }
}

/// A single sample entry in a trun box.
///
/// `None` fields mean the value was not present in the per-sample trun data.
/// After decode, callers should resolve `None` against tfhd defaults
/// (`default_sample_duration`, `default_sample_size`, `default_sample_flags`)
/// before using the values. The encoder backfills unresolved `None` with `0`
/// as a last resort to avoid silently dropping fields that other entries set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrunEntry {
    pub duration: Option<u32>,
    pub size: Option<u32>,
    pub flags: Option<u32>,
    /// The composition offset: unsigned in a version 0 box, signed in version 1.
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

        let mut first_sample_flags = match ext.first_sample_flags {
            true => u32::decode(buf)?.into(),
            false => None,
        };

        let fields = [
            ext.sample_duration,
            ext.sample_size,
            ext.sample_flags,
            ext.sample_cts,
        ]
        .into_iter()
        .filter(|&field| field)
        .count();

        // A run without per-sample fields has zero bytes per sample, so store it as a count
        // instead of allocating from the untrusted sample_count.
        if fields == 0 {
            return Ok(Trun {
                data_offset,
                samples: TrunSamples::Inherited {
                    count: sample_count,
                    first_flags: first_sample_flags,
                },
            });
        }

        // Otherwise every sample takes at least 4 bytes, so the remaining bytes bound the allocation.
        let capacity = (sample_count as usize).min(buf.remaining() / (4 * fields));
        let mut entries = Vec::with_capacity(capacity);

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
            let cts = match (ext.sample_cts, ext.version) {
                (false, _) => None,
                (true, TrunVersion::V0) => Some(u32::decode(buf)?.into()),
                (true, TrunVersion::V1) => Some(i32::decode(buf)?.into()),
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
            samples: TrunSamples::Explicit(entries),
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<TrunExt> {
        let entries = match &self.samples {
            TrunSamples::Inherited { count, first_flags } => {
                let ext = TrunExt {
                    version: TrunVersion::V1,
                    data_offset: self.data_offset.is_some(),
                    first_sample_flags: first_flags.is_some(),
                    ..Default::default()
                };

                count.encode(buf)?;
                self.data_offset.encode(buf)?;
                first_flags.encode(buf)?;

                return Ok(ext);
            }
            TrunSamples::Explicit(entries) => entries,
        };

        let any_flags = entries.iter().any(|s| s.flags.is_some());
        let first_only_flags = any_flags
            && entries.first().is_some_and(|s| s.flags.is_some())
            && entries.iter().skip(1).all(|s| s.flags.is_none());

        // Use per-sample flags when any entry has flags and it's not the first-only pattern.
        // None entries are backfilled with 0 to avoid silently dropping flags.
        let sample_flags = any_flags && !first_only_flags;

        // Keep version 1's signed offsets unless one only fits version 0's unsigned range.
        let mut cts = entries.iter().filter_map(|s| s.cts);
        let version = if cts.clone().all(|cts| i32::try_from(cts).is_ok()) {
            TrunVersion::V1
        } else if cts.all(|cts| u32::try_from(cts).is_ok()) {
            TrunVersion::V0
        } else {
            return Err(Error::InvalidCombination(
                "trun composition offsets fit neither i32 nor u32",
            ));
        };

        let ext = TrunExt {
            version,
            data_offset: self.data_offset.is_some(),
            first_sample_flags: first_only_flags,

            // Use the field if any entry has it set. None entries are backfilled
            // with 0 during encoding to avoid silently dropping fields on
            // decode→encode roundtrips (entries that inherited defaults from tfhd
            // have None after decode).
            sample_duration: entries.iter().any(|s| s.duration.is_some()),
            sample_size: entries.iter().any(|s| s.size.is_some()),
            sample_flags,
            sample_cts: entries.iter().any(|s| s.cts.is_some()),
        };

        (entries.len() as u32).encode(buf)?;

        self.data_offset.encode(buf)?;
        if ext.first_sample_flags {
            entries[0].flags.unwrap().encode(buf)?;
        }

        for entry in entries {
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
                // Range-checked against the version above.
                let cts = entry.cts.unwrap_or(0);
                match version {
                    TrunVersion::V0 => (cts as u32).encode(buf)?,
                    TrunVersion::V1 => (cts as i32).encode(buf)?,
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
            samples: TrunSamples::Explicit(vec![
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
            ]),
        };

        let mut buf = Vec::new();
        trun.encode(&mut buf).expect("encode");

        let decoded = Trun::decode(&mut &buf[..]).expect("decode");
        assert_eq!(decoded.data_offset, Some(100));
        let decoded: Vec<_> = decoded.samples.iter().collect();

        // entry[0] must have the keyframe flags from first_sample_flags
        assert_eq!(decoded[0].flags, Some(0x02000000));
        // entries[1..N] must have None (they use default_sample_flags from tfhd)
        assert_eq!(decoded[1].flags, None);
        assert_eq!(decoded[2].flags, None);
        assert_eq!(decoded.len(), 3);
    }

    /// When multiple entries have explicit flags (not just the first),
    /// the encoder must use per-sample flags, not first_sample_flags.
    #[test]
    fn mixed_flags_uses_per_sample() {
        let trun = Trun {
            data_offset: Some(100),
            samples: TrunSamples::Explicit(vec![
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
            ]),
        };

        let mut buf = Vec::new();
        trun.encode(&mut buf).expect("encode");

        let decoded = Trun::decode(&mut &buf[..]).expect("decode");
        let decoded: Vec<_> = decoded.samples.iter().collect();

        // Mixed Some/None: encoder backfills None with 0 and emits per-sample flags.
        assert_eq!(decoded[0].flags, Some(0x02000000));
        assert_eq!(decoded[1].flags, Some(0x01010000));
        assert_eq!(decoded[2].flags, Some(0)); // was None, backfilled to 0
    }

    /// When all entries have explicit flags, per-sample flags are used.
    #[test]
    fn all_flags_roundtrip() {
        let trun = Trun {
            data_offset: Some(100),
            samples: TrunSamples::Explicit(vec![
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
            ]),
        };

        let mut buf = Vec::new();
        trun.encode(&mut buf).expect("encode");

        let decoded = Trun::decode(&mut &buf[..]).expect("decode");
        let decoded: Vec<_> = decoded.samples.iter().collect();

        assert_eq!(decoded[0].flags, Some(0x02000000));
        assert_eq!(decoded[1].flags, Some(0x01010000));
    }

    /// Entries with None duration (inherited from tfhd default_sample_duration)
    /// must not cause the duration field to be dropped entirely on re-encode.
    #[test]
    fn duration_backfill_roundtrip() {
        let trun = Trun {
            data_offset: Some(100),
            samples: TrunSamples::Explicit(vec![
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
            ]),
        };

        let mut buf = Vec::new();
        trun.encode(&mut buf).expect("encode");

        let decoded = Trun::decode(&mut &buf[..]).expect("decode");
        let decoded: Vec<_> = decoded.samples.iter().collect();

        assert_eq!(decoded[0].duration, Some(512));
        // None backfilled to 0 during encode
        assert_eq!(decoded[1].duration, Some(0));
    }

    /// Entries with None size (inherited from tfhd default_sample_size)
    /// must not cause the size field to be dropped entirely on re-encode.
    #[test]
    fn size_backfill_roundtrip() {
        let trun = Trun {
            data_offset: Some(100),
            samples: TrunSamples::Explicit(vec![
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
            ]),
        };

        let mut buf = Vec::new();
        trun.encode(&mut buf).expect("encode");

        let decoded = Trun::decode(&mut &buf[..]).expect("decode");
        let decoded: Vec<_> = decoded.samples.iter().collect();

        assert_eq!(decoded[0].size, Some(1000));
        // None backfilled to 0 during encode
        assert_eq!(decoded[1].size, Some(0));
    }

    /// When all entries have None for a field, the flag is not set and
    /// the field is omitted entirely (no unnecessary bytes written).
    #[test]
    fn all_none_fields_omitted() {
        let trun = Trun {
            data_offset: Some(100),
            samples: TrunSamples::Explicit(vec![
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
            ]),
        };

        let mut buf = Vec::new();
        trun.encode(&mut buf).expect("encode");

        let decoded = Trun::decode(&mut &buf[..]).expect("decode");
        let decoded: Vec<_> = decoded.samples.iter().collect();

        assert_eq!(decoded[0].duration, None);
        assert_eq!(decoded[0].size, None);
        assert_eq!(decoded[0].flags, None);
        assert_eq!(decoded[0].cts, None);
    }

    /// A run without per-sample fields decodes as a count, not one entry per sample.
    #[test]
    fn inherited_roundtrip() {
        for first_flags in [None, Some(0x02000000)] {
            let trun = Trun {
                data_offset: Some(100),
                samples: TrunSamples::Inherited {
                    count: u32::MAX,
                    first_flags,
                },
            };

            let mut buf = Vec::new();
            trun.encode(&mut buf).expect("encode");

            // size, fourcc, version/flags, sample_count, data_offset, and the optional first flags.
            let expected = 20 + first_flags.map_or(0, |_| 4);
            assert_eq!(buf.len(), expected);

            let decoded = Trun::decode(&mut &buf[..]).expect("decode");
            assert_eq!(decoded, trun);
            assert_eq!(decoded.samples.len(), u32::MAX as usize);

            let mut entries = decoded.samples.iter();
            assert_eq!(entries.len(), u32::MAX as usize);
            assert_eq!(
                entries.next(),
                Some(TrunEntry {
                    flags: first_flags,
                    ..Default::default()
                })
            );
            assert_eq!(entries.next(), Some(TrunEntry::default()));
            assert_eq!(entries.len(), u32::MAX as usize - 2);

            // Skipping into the run takes constant time.
            assert_eq!(
                entries.nth(u32::MAX as usize - 3),
                Some(TrunEntry::default())
            );
            assert_eq!(entries.next(), None);

            let last = u32::MAX as usize - 1;
            assert_eq!(decoded.samples.get(0), decoded.samples.iter().next());
            assert_eq!(decoded.samples.get(last), Some(TrunEntry::default()));
            assert_eq!(decoded.samples.get(last + 1), None);
        }
    }

    /// Explicit entries without any field encode as an inherited run.
    #[test]
    fn explicit_without_fields_decodes_inherited() {
        let trun = Trun {
            data_offset: None,
            samples: TrunSamples::Explicit(vec![
                TrunEntry {
                    flags: Some(0x02000000),
                    ..Default::default()
                },
                TrunEntry::default(),
            ]),
        };

        let mut buf = Vec::new();
        trun.encode(&mut buf).expect("encode");

        let decoded = Trun::decode(&mut &buf[..]).expect("decode");
        assert_eq!(
            decoded.samples,
            TrunSamples::Inherited {
                count: 2,
                first_flags: Some(0x02000000),
            }
        );
        assert!(decoded.samples.iter().eq(trun.samples.iter()));
    }

    /// `get` and `nth` agree with stepping through the run in both forms.
    #[test]
    fn random_access() {
        let inherited = TrunSamples::Inherited {
            count: 3,
            first_flags: Some(0x02000000),
        };
        let explicit = TrunSamples::Explicit(vec![
            TrunEntry {
                size: Some(1),
                ..Default::default()
            },
            TrunEntry {
                size: Some(2),
                ..Default::default()
            },
        ]);

        for samples in [inherited, explicit] {
            let all: Vec<_> = samples.iter().collect();
            for n in 0..=all.len() {
                assert_eq!(samples.get(n), all.get(n).copied());
                assert_eq!(samples.iter().nth(n), all.get(n).copied());
            }
        }
    }

    fn cts_entries(cts: &[i64]) -> Trun {
        Trun {
            data_offset: None,
            samples: cts
                .iter()
                .map(|&cts| TrunEntry {
                    cts: Some(cts),
                    ..Default::default()
                })
                .collect::<Vec<_>>()
                .into(),
        }
    }

    /// Composition offsets pick the version that represents them: signed (1) by default,
    /// unsigned (0) only when an offset exceeds i32.
    #[test]
    fn cts_version_roundtrip() {
        for (cts, version) in [
            (vec![0, 1024, -512], 1),
            (vec![0, 1 << 31, u32::MAX as i64], 0),
        ] {
            let trun = cts_entries(&cts);
            let mut buf = Vec::new();
            trun.encode(&mut buf).expect("encode");
            assert_eq!(buf[8], version);

            let decoded = Trun::decode(&mut &buf[..]).expect("decode");
            assert!(decoded.samples.iter().map(|e| e.cts.unwrap()).eq(cts));
        }
    }

    /// A version 0 offset above i32::MAX decodes as unsigned, not negative.
    #[test]
    fn cts_version_0_is_unsigned() {
        let mut buf = Vec::new();
        cts_entries(&[-1]).encode(&mut buf).expect("encode");
        assert_eq!(buf[8], 1);
        buf[8] = 0;

        let decoded = Trun::decode(&mut &buf[..]).expect("decode");
        assert_eq!(decoded.samples.get(0).unwrap().cts, Some(u32::MAX as i64));
    }

    #[test]
    fn cts_out_of_range_is_refused() {
        for cts in [vec![-1, 1 << 31], vec![1 << 32], vec![i32::MIN as i64 - 1]] {
            assert!(matches!(
                cts_entries(&cts).encode(&mut Vec::new()),
                Err(Error::InvalidCombination(_))
            ));
        }
    }

    /// A sample_count larger than the per-sample data fails without reserving for it.
    #[test]
    fn explicit_truncated() {
        let mut buf = Vec::new();
        Trun {
            data_offset: None,
            samples: TrunSamples::Explicit(vec![TrunEntry {
                size: Some(1),
                ..Default::default()
            }]),
        }
        .encode(&mut buf)
        .expect("encode");

        // Claim u32::MAX samples while carrying one.
        buf[12..16].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(Trun::decode(&mut &buf[..]).is_err());
    }
}
