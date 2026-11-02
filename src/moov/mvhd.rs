use crate::*;

ext! {
    name: Mvhd,
    versions: [0,1],
    flags: {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mvhd {
    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,

    pub rate: FixedPoint<u16>,
    pub volume: FixedPoint<u8>,

    pub matrix: Matrix,
    pub next_track_id: u32,
}

impl AtomExt for Mvhd {
    const KIND_EXT: FourCC = FourCC::new(b"mvhd");

    type Ext = MvhdExt;

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: MvhdExt) -> Result<Self> {
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
                // All 1s means unknown; keep it all 1s in 64 bits.
                match u32::decode(buf)? {
                    u32::MAX => u64::MAX,
                    duration => duration as u64,
                },
            ),
        };

        let rate = FixedPoint::decode(buf)?;
        let volume = FixedPoint::decode(buf)?;

        u16::decode(buf)?; // reserved
        u64::decode(buf)?; // reserved

        let matrix = Matrix::decode(buf)?;

        <[u8; 24]>::decode(buf)?; // pre_defined = 0

        let next_track_id = u32::decode(buf)?;

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

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<MvhdExt> {
        self.creation_time.encode(buf)?;
        self.modification_time.encode(buf)?;
        self.timescale.encode(buf)?;
        self.duration.encode(buf)?;

        self.rate.encode(buf)?;
        self.volume.encode(buf)?;

        0u16.encode(buf)?; // reserved
        0u64.encode(buf)?; // reserved

        self.matrix.encode(buf)?;

        [0u8; 24].encode(buf)?; // pre_defined = 0

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
            rate: Default::default(),
            matrix: Default::default(),
            volume: Default::default(),
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
            rate: 1.into(),
            volume: 1.into(),
            matrix: Matrix::default(),
            next_track_id: 1,
        };

        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
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
            rate: 1.into(),
            volume: 1.into(),
            matrix: Matrix::default(),
            next_track_id: 1,
        };

        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let output = Mvhd::decode(&mut buf).unwrap();
        assert_eq!(output, expected);
    }

    // All 1s is an unknown duration (ISO/IEC 14496-12 §8.2.2.3) at either width.
    #[test]
    fn test_mvhd_v0_unknown_duration_reencode() {
        let raw = [
            &[0x00, 0x00, 0x00, 0x6C][..],
            b"mvhd",
            &[0x00, 0x00, 0x00, 0x00], // version 0, flags 0
            &[0x00, 0x00, 0x00, 0x64], // creation_time = 100
            &[0x00, 0x00, 0x00, 0xC8], // modification_time = 200
            &[0x00, 0x00, 0x03, 0xE8], // timescale = 1000
            &[0xFF, 0xFF, 0xFF, 0xFF], // duration = unknown
            &[0x00, 0x01, 0x00, 0x00], // rate = 1.0
            &[0x01, 0x00, 0x00, 0x00], // volume = 1.0, reserved
            &[0; 8],                   // reserved
            &[0x00, 0x01, 0x00, 0x00, 0, 0, 0, 0, 0, 0, 0, 0], // matrix
            &[0, 0, 0, 0, 0x00, 0x01, 0x00, 0x00, 0, 0, 0, 0],
            &[0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x00, 0x00, 0x00],
            &[0; 24],                  // pre_defined
            &[0x00, 0x00, 0x00, 0x02], // next_track_id = 2
        ]
        .concat();
        let decoded = Mvhd::decode(&mut raw.as_slice()).unwrap();
        let mut buf = Vec::new();
        decoded.encode(&mut buf).unwrap();
        assert_eq!(buf[8], 1, "re-encoded as version 1");
        assert_eq!(buf[32..40], [0xFF; 8], "duration is still unknown");
        assert_eq!(decoded.duration, u64::MAX);

        // One below all 1s is still a finite duration.
        let mut finite = raw.to_vec();
        finite[27] = 0xFE;
        let decoded = Mvhd::decode(&mut finite.as_slice()).unwrap();
        assert_eq!(decoded.duration, u32::MAX as u64 - 1);
    }

    #[test]
    fn test_mvhd64_duration_u32_max() {
        let expected = Mvhd {
            duration: u32::MAX as u64,
            ..Default::default()
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let decoded = Mvhd::decode(&mut buf.as_slice()).unwrap();
        assert_eq!(decoded, expected);
    }
}
