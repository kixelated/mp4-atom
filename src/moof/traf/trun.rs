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

            // TODO error if these are not all the same
            sample_duration: self.entries.iter().all(|s| s.duration.is_some()),
            sample_size: self.entries.iter().all(|s| s.size.is_some()),
            sample_flags,
            sample_cts: self.entries.iter().all(|s| s.cts.is_some()),
        };

        (self.entries.len() as u32).encode(buf)?;

        self.data_offset.encode(buf)?;
        if ext.first_sample_flags {
            self.entries[0].flags.unwrap().encode(buf)?;
        }

        for entry in &self.entries {
            ext.sample_duration.then_some(entry.duration).encode(buf)?;
            ext.sample_size.then_some(entry.size).encode(buf)?;
            if ext.sample_flags {
                Some(Some(entry.flags.unwrap_or(0))).encode(buf)?;
            }
            ext.sample_cts.then_some(entry.cts).encode(buf)?;
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
}
