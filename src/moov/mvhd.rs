use crate::*;

ext! {
    name: Mvhd,
    versions: [0,1],
    flags: {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mvhd {
    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,

    pub rate: Ratio<u16>,
    pub volume: Ratio<u8>,

    pub matrix: Matrix,
    pub next_track_id: u32,
}

impl AtomExt for Mvhd {
    const KIND: FourCC = FourCC::new(b"mvhd");

    type Ext = MvhdExt;

    fn decode_atom(buf: &mut Buf, ext: MvhdExt) -> Result<Self> {
        let (creation_time, modification_time, timescale, duration) = match ext.version {
            MvhdVersion::V1 => (
                u64::decode(buf)?,
                u64::decode(buf)?,
                u32::decode(buf)?,
                u64::decode(buf)?,
            ),
            MvhdVersion::V0 => (
                u32::decode(buf)? as u64,
                u32::decode(buf)? as u64,
                u32::decode(buf)?,
                u32::decode(buf)? as u64,
            ),
        };

        let rate = buf.decode()?;
        let volume = buf.decode()?;

        buf.skip(2 + 8)?; // reserved = 0

        let matrix = buf.decode()?;

        buf.skip(24)?; // pre_defined = 0

        let next_track_id = buf.decode()?;

        Ok(Mvhd {
            creation_time,
            modification_time,
            timescale,
            duration,
            rate,
            volume,
            matrix,
            next_track_id,
        })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<MvhdExt> {
        self.creation_time.encode(buf)?;
        self.modification_time.encode(buf)?;
        self.timescale.encode(buf)?;
        self.duration.encode(buf)?;

        self.rate.encode(buf)?;
        self.volume.encode(buf)?;

        buf.zero(2 + 8)?; // reserved = 0

        self.matrix.encode(buf)?;

        buf.zero(24)?; // pre_defined = 0

        self.next_track_id.encode(buf)?;

        Ok(MvhdVersion::V1.into())
    }
}

impl Default for Mvhd {
    fn default() -> Self {
        Mvhd {
            creation_time: 0,
            modification_time: 0,
            timescale: 1000,
            duration: 0,
            rate: Ratio::default(),
            matrix: Matrix::default(),
            volume: Ratio::default(),
            next_track_id: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mvhd32() {
        let expected = Mvhd {
            creation_time: 100,
            modification_time: 200,
            timescale: 1000,
            duration: 634634,
            rate: Ratio::new(1, 1),
            volume: Ratio::new(1, 1),
            matrix: Matrix::default(),
            next_track_id: 1,
        };

        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Mvhd::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_mvhd64() {
        let expected = Mvhd {
            creation_time: 100,
            modification_time: 200,
            timescale: 1000,
            duration: 634634,
            rate: Ratio::new(1, 1),
            volume: Ratio::new(1, 1),
            matrix: Matrix::default(),
            next_track_id: 1,
        };

        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let output = Mvhd::decode(&mut buf).unwrap();
        assert_eq!(output, expected);
    }
}
