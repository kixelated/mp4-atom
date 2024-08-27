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
    pub entires: Vec<TrunEntry>,
}

pub struct TrunEntry {
    pub duration: Option<u32>,
    pub size: Option<u32>,
    pub flags: Option<u32>,
    pub cts: Option<i32>,
}

impl AtomExt for Trun {
    const KIND: FourCC = FourCC::new(b"trun");

    type Ext = TrunExt;

    fn decode_atom(buf: &mut Buf, ext: TrunExt) -> Result<Self> {
        let sample_count = u32::decode(buf)?;
        let data_offset = ext.data_offset.then(buf.decode()).transpose()?;
        let first_sample_flags = ext.first_sample_flags.then(buf.decode()).transpose()?;

        let mut entires = Vec::new();

        // TODO this is undoubtedly wrong
        for _ in 0..sample_count {
            let duration = ext.sample_duration.then(buf.decode()).transpose()?;
            let size = ext.sample_size.then(buf.decode()).transpose()?;
            let sample_flags = ext.sample_flags.then(buf.decode()).transpose()?;
            let cts = ext.sample_cts.then(buf.decode()).transpose()?;

            entires.push(TrunEntry {
                duration,
                size,
                flags: sample_flags,
                cts,
            });
        }

        Ok(Trun {
            data_offset,
            entires,
        })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<TrunExt> {
        let ext = TrunExt {
            version: TrunVersion::V1,
            data_offset: self.data_offset.is_some(),
            first_sample_flags: false,

            // TODO error if these are not all the same
            sample_duration: self.entires.iter().all(|s| s.duration.is_some()),
            sample_size: self.entires.iter().all(|s| s.size.is_some()),
            sample_flags: self.entires.iter().all(|s| s.flags.is_some()),
            sample_cts: self.entires.iter().all(|s| s.cts.is_some()),
        };

        (self.entires.len() as u32).encode(buf)?;

        self.data_offset.encode(buf)?;
        0u32.encode(buf)?; // TODO first sample flags

        for entry in &self.entires {
            ext.sample_duration.then(entry.duration).encode(buf)?;
            ext.sample_size.then(entry.size).encode(buf)?;
            ext.sample_flags.then(entry.flags).encode(buf)?;
            ext.sample_cts.then(entry.cts).encode(buf)?;
        }

        Ok(ext)
    }
}
