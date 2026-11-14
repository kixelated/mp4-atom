use crate::*;

ext! {
    name: Mdhd,
    versions: [0, 1],
    flags: {}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mdhd {
    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,
    pub language: String,
}

impl AtomExt for Mdhd {
    type Ext = MdhdExt;

    const KIND_EXT: FourCC = FourCC::new(b"mdhd");

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: MdhdExt) -> Result<Self> {
        let (creation_time, modification_time, timescale, duration) = match ext.version {
            MdhdVersion::V1 => (
                u64::decode(buf)?,
                u64::decode(buf)?,
                u32::decode(buf)?,
                u64::decode(buf)?,
            ),
            MdhdVersion::V0 => (
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

        let language_code = u16::decode(buf)?;
        let language = language_string(language_code);

        u16::decode(buf)?; // pre-defined

        Ok(Mdhd {
            creation_time,
            modification_time,
            timescale,
            duration,
            language,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<MdhdExt> {
        self.creation_time.encode(buf)?;
        self.modification_time.encode(buf)?;
        self.timescale.encode(buf)?;
        self.duration.encode(buf)?;

        let language_code = language_code(&self.language);
        (language_code).encode(buf)?;
        0u16.encode(buf)?; // pre-defined

        Ok(MdhdVersion::V1.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mdhd32() {
        let expected = Mdhd {
            creation_time: 100,
            modification_time: 200,
            timescale: 48000,
            duration: 30439936,
            language: String::from("und"),
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Mdhd::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_mdhd64() {
        let expected = Mdhd {
            creation_time: 100,
            modification_time: 200,
            timescale: 48000,
            duration: 30439936,
            language: String::from("eng"),
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Mdhd::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    // All 1s is an unknown duration (ISO/IEC 14496-12 §8.4.2.3) at either width.
    #[test]
    fn test_mdhd_v0_unknown_duration_reencode() {
        let raw: &[u8] = &[
            0x00, 0x00, 0x00, 0x20, b'm', b'd', b'h', b'd', //
            0x00, 0x00, 0x00, 0x00, // version 0, flags 0
            0x00, 0x00, 0x00, 0x64, // creation_time = 100
            0x00, 0x00, 0x00, 0xC8, // modification_time = 200
            0x00, 0x00, 0xBB, 0x80, // timescale = 48000
            0xFF, 0xFF, 0xFF, 0xFF, // duration = unknown
            0x55, 0xC4, 0x00, 0x00, // language = "und", pre_defined
        ];
        let decoded = Mdhd::decode(&mut &raw[..]).unwrap();
        let mut buf = Vec::new();
        decoded.encode(&mut buf).unwrap();
        assert_eq!(buf[8], 1, "re-encoded as version 1");
        assert_eq!(buf[32..40], [0xFF; 8], "duration is still unknown");
        assert_eq!(decoded.duration, u64::MAX);

        // One below all 1s is still a finite duration.
        let mut finite = raw.to_vec();
        finite[27] = 0xFE;
        let decoded = Mdhd::decode(&mut finite.as_slice()).unwrap();
        assert_eq!(decoded.duration, u32::MAX as u64 - 1);
    }

    #[test]
    fn test_mdhd64_duration_u32_max() {
        let expected = Mdhd {
            timescale: 48000,
            duration: u32::MAX as u64,
            language: String::from("und"),
            ..Default::default()
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let decoded = Mdhd::decode(&mut buf.as_slice()).unwrap();
        assert_eq!(decoded, expected);
    }
}
