use crate::*;

ext! {
    name: Tkhd,
    versions: [0, 1],
    flags: {
        track_enabled = 0,
        track_in_movie = 1,
        track_in_preview = 2,
        track_size_is_aspect_ratio = 3,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tkhd {
    pub creation_time: u64,
    pub modification_time: u64,
    pub track_id: u32,
    pub duration: u64,
    pub layer: u16,
    pub alternate_group: u16,
    pub enabled: bool,
    pub in_movie: bool,
    /// When set, `width`/`height` express the track's suggested aspect ratio
    /// rather than pixel dimensions (ISO/IEC 14496-12:2022 §8.3.2.3).
    pub size_is_aspect_ratio: bool,

    pub volume: FixedPoint<u8>,
    pub matrix: Matrix,

    pub width: FixedPoint<u16>,
    pub height: FixedPoint<u16>,
}

impl AtomExt for Tkhd {
    const KIND_EXT: FourCC = FourCC::new(b"tkhd");

    type Ext = TkhdExt;

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: TkhdExt) -> Result<Self> {
        let (creation_time, modification_time, track_id, _, duration) = match ext.version {
            TkhdVersion::V1 => (
                u64::decode(buf)?,
                u64::decode(buf)?,
                u32::decode(buf)?,
                u32::decode(buf)?,
                u64::decode(buf)?,
            ),
            TkhdVersion::V0 => (
                u32::decode(buf)? as u64,
                u32::decode(buf)? as u64,
                u32::decode(buf)?,
                u32::decode(buf)?,
                // All 1s means unknown; keep it all 1s in 64 bits.
                match u32::decode(buf)? {
                    u32::MAX => u64::MAX,
                    duration => duration as u64,
                },
            ),
        };

        u64::decode(buf)?; // reserved
        let layer = u16::decode(buf)?;
        let alternate_group = u16::decode(buf)?;
        let volume = FixedPoint::decode(buf)?;

        u16::decode(buf)?; // reserved
        let matrix = Matrix::decode(buf)?;
        let width = FixedPoint::decode(buf)?;
        let height = FixedPoint::decode(buf)?;

        Ok(Tkhd {
            creation_time,
            modification_time,
            track_id,
            duration,
            layer,
            alternate_group,
            volume,
            matrix,
            width,
            height,
            enabled: ext.track_enabled,
            in_movie: ext.track_in_movie,
            size_is_aspect_ratio: ext.track_size_is_aspect_ratio,
            // track_in_preview has no assigned meaning and is ignored on read
            // per ISO/IEC 14496-12:2022 §8.3.2.3.
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<TkhdExt> {
        self.creation_time.encode(buf)?;
        self.modification_time.encode(buf)?;
        self.track_id.encode(buf)?;
        0u32.encode(buf)?; // reserved
        self.duration.encode(buf)?;

        0u64.encode(buf)?; // reserved
        self.layer.encode(buf)?;
        self.alternate_group.encode(buf)?;
        self.volume.encode(buf)?;
        0u16.encode(buf)?; // reserved
        self.matrix.encode(buf)?;

        self.width.encode(buf)?;
        self.height.encode(buf)?;

        Ok(TkhdExt {
            version: TkhdVersion::V1,
            track_enabled: self.enabled,
            track_in_movie: self.in_movie,
            // track_in_preview has no assigned meaning; per ISO/IEC
            // 14496-12:2022 §8.3.2.3 writers should mirror track_in_movie.
            track_in_preview: self.in_movie,
            track_size_is_aspect_ratio: self.size_is_aspect_ratio,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Matrix {
    pub a: i32,
    pub b: i32,
    pub u: i32,
    pub c: i32,
    pub d: i32,
    pub v: i32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
}

impl Decode for Matrix {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Self {
            a: i32::decode(buf)?,
            b: i32::decode(buf)?,
            u: i32::decode(buf)?,
            c: i32::decode(buf)?,
            d: i32::decode(buf)?,
            v: i32::decode(buf)?,
            x: i32::decode(buf)?,
            y: i32::decode(buf)?,
            w: i32::decode(buf)?,
        })
    }
}

impl Encode for Matrix {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.a.encode(buf)?;
        self.b.encode(buf)?;
        self.u.encode(buf)?;
        self.c.encode(buf)?;
        self.d.encode(buf)?;
        self.v.encode(buf)?;
        self.x.encode(buf)?;
        self.y.encode(buf)?;
        self.w.encode(buf)
    }
}

impl std::fmt::Display for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:#x} {:#x} {:#x} {:#x} {:#x} {:#x} {:#x} {:#x} {:#x}",
            self.a, self.b, self.u, self.c, self.d, self.v, self.x, self.y, self.w
        )
    }
}

impl Default for Matrix {
    fn default() -> Self {
        Self {
            // unity matrix according to ISO/IEC 14496-12:2005(E)
            a: 0x00010000,
            b: 0,
            u: 0,
            c: 0,
            d: 0x00010000,
            v: 0,
            x: 0,
            y: 0,
            w: 0x40000000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tkhd32() {
        let expected = Tkhd {
            creation_time: 100,
            modification_time: 200,
            track_id: 1,
            duration: 634634,
            layer: 0,
            alternate_group: 0,
            volume: 1.into(),
            matrix: Matrix::default(),
            width: 512.into(),
            height: 288.into(),
            enabled: true,
            in_movie: true,
            size_is_aspect_ratio: false,
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Tkhd::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_tkhd64() {
        let expected = Tkhd {
            creation_time: 100,
            modification_time: 200,
            track_id: 1,
            duration: 634634,
            layer: 0,
            alternate_group: 0,
            volume: 1.into(),
            matrix: Matrix::default(),
            width: 512.into(),
            height: 288.into(),
            enabled: true,
            in_movie: true,
            size_is_aspect_ratio: false,
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Tkhd::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    // All 1s is an unknown duration (ISO/IEC 14496-12 §8.3.2.3) at either width.
    #[test]
    fn test_tkhd_v0_unknown_duration_reencode() {
        let raw = [
            &[0x00, 0x00, 0x00, 0x5C][..],
            b"tkhd",
            &[0x00, 0x00, 0x00, 0x03], // version 0, flags: enabled, in_movie
            &[0x00, 0x00, 0x00, 0x64], // creation_time = 100
            &[0x00, 0x00, 0x00, 0xC8], // modification_time = 200
            &[0x00, 0x00, 0x00, 0x01], // track_id = 1
            &[0x00, 0x00, 0x00, 0x00], // reserved
            &[0xFF, 0xFF, 0xFF, 0xFF], // duration = unknown
            &[0; 8],                   // reserved
            &[0; 4],                   // layer, alternate_group
            &[0x01, 0x00, 0x00, 0x00], // volume = 1.0, reserved
            &[0x00, 0x01, 0x00, 0x00, 0, 0, 0, 0, 0, 0, 0, 0], // matrix
            &[0, 0, 0, 0, 0x00, 0x01, 0x00, 0x00, 0, 0, 0, 0],
            &[0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x00, 0x00, 0x00],
            &[0x02, 0x00, 0x00, 0x00], // width = 512
            &[0x01, 0x20, 0x00, 0x00], // height = 288
        ]
        .concat();
        let decoded = Tkhd::decode(&mut raw.as_slice()).unwrap();
        let mut buf = Vec::new();
        decoded.encode(&mut buf).unwrap();
        assert_eq!(buf[8], 1, "re-encoded as version 1");
        assert_eq!(buf[36..44], [0xFF; 8], "duration is still unknown");
        assert_eq!(decoded.duration, u64::MAX);

        // One below all 1s is still a finite duration.
        let mut finite = raw.to_vec();
        finite[31] = 0xFE;
        let decoded = Tkhd::decode(&mut finite.as_slice()).unwrap();
        assert_eq!(decoded.duration, u32::MAX as u64 - 1);
    }

    #[test]
    fn test_tkhd64_duration_u32_max() {
        let expected = Tkhd {
            track_id: 1,
            duration: u32::MAX as u64,
            enabled: true,
            in_movie: true,
            ..Default::default()
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let decoded = Tkhd::decode(&mut buf.as_slice()).unwrap();
        assert_eq!(decoded, expected);
    }
}
