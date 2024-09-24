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
pub struct Trun {
    pub data_offset: Option<i32>,
    pub entries: Vec<TrunEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TrunEntry {
    pub duration: Option<u32>,
    pub size: Option<u32>,
    pub flags: Option<u32>,
    pub cts: Option<i32>,
}

impl AtomExt for Trun {
    const KIND_EXT: FourCC = FourCC::new(b"trun");

    type Ext = TrunExt;

    fn decode_body_ext(buf: &mut Bytes, ext: TrunExt) -> Result<Self> {
        let sample_count = u32::decode(buf)?;
        let data_offset = match ext.data_offset {
            true => i32::decode(buf)?.into(),
            false => None,
        };

        let mut first_sample_flags = match ext.first_sample_flags {
            true => u32::decode(buf)?.into(),
            false => None,
        };

        let mut entries = Vec::new();

        // TODO this is undoubtedly wrong
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

    fn encode_body_ext(&self, buf: &mut BytesMut) -> Result<TrunExt> {
        let ext = TrunExt {
            version: TrunVersion::V1,
            data_offset: self.data_offset.is_some(),
            first_sample_flags: false,

            // TODO error if these are not all the same
            sample_duration: self.entries.iter().all(|s| s.duration.is_some()),
            sample_size: self.entries.iter().all(|s| s.size.is_some()),
            sample_flags: self.entries.iter().all(|s| s.flags.is_some()),
            sample_cts: self.entries.iter().all(|s| s.cts.is_some()),
        };

        (self.entries.len() as u32).encode(buf)?;

        self.data_offset.encode(buf)?;
        0u32.encode(buf)?; // TODO first sample flags

        for entry in &self.entries {
            ext.sample_duration.then_some(entry.duration).encode(buf)?;
            ext.sample_size.then_some(entry.size).encode(buf)?;
            ext.sample_flags.then_some(entry.flags).encode(buf)?;
            ext.sample_cts.then_some(entry.cts).encode(buf)?;
        }

        Ok(ext)
    }
}
