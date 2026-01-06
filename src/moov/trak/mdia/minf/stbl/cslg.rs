use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cslg {
    pub composition_to_dts_shift: i64,
    pub least_decode_to_display_shift: i64,
    pub greatest_decode_to_display_delta: i64,
    pub composition_start_time: i64,
    pub composition_end_time: i64,
}

ext! {
    name: Cslg,
    versions: [0, 1],
    flags: {}
}

impl AtomExt for Cslg {
    type Ext = CslgExt;

    const KIND_EXT: FourCC = FourCC::new(b"cslg");

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: CslgExt) -> Result<Self> {
        if ext.version == CslgVersion::V0 {
            Ok(Cslg {
                composition_to_dts_shift: i32::decode(buf)?.into(),
                least_decode_to_display_shift: i32::decode(buf)?.into(),
                greatest_decode_to_display_delta: i32::decode(buf)?.into(),
                composition_start_time: i32::decode(buf)?.into(),
                composition_end_time: i32::decode(buf)?.into(),
            })
        } else {
            Ok(Cslg {
                composition_to_dts_shift: i64::decode(buf)?,
                least_decode_to_display_shift: i64::decode(buf)?,
                greatest_decode_to_display_delta: i64::decode(buf)?,
                composition_start_time: i64::decode(buf)?,
                composition_end_time: i64::decode(buf)?,
            })
        }
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<CslgExt> {
        if self.composition_to_dts_shift <= i32::MAX.into()
            && self.composition_to_dts_shift >= i32::MIN.into()
            && self.least_decode_to_display_shift <= i32::MAX.into()
            && self.least_decode_to_display_shift >= i32::MIN.into()
            && self.greatest_decode_to_display_delta <= i32::MAX.into()
            && self.greatest_decode_to_display_delta >= i32::MIN.into()
            && self.composition_start_time <= i32::MAX.into()
            && self.composition_start_time >= i32::MIN.into()
            && self.composition_end_time <= i32::MAX.into()
            && self.composition_end_time >= i32::MIN.into()
        {
            (self.composition_to_dts_shift as i32).encode(buf)?;
            (self.least_decode_to_display_shift as i32).encode(buf)?;
            (self.greatest_decode_to_display_delta as i32).encode(buf)?;
            (self.composition_start_time as i32).encode(buf)?;
            (self.composition_end_time as i32).encode(buf)?;
            Ok(CslgExt {
                version: CslgVersion::V0,
            })
        } else {
            self.composition_to_dts_shift.encode(buf)?;
            self.least_decode_to_display_shift.encode(buf)?;
            self.greatest_decode_to_display_delta.encode(buf)?;
            self.composition_start_time.encode(buf)?;
            self.composition_end_time.encode(buf)?;
            Ok(CslgExt {
                version: CslgVersion::V1,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // From MPEG file format conformance suite, 17_negative_ctso.mp4
    const ENCODED_CSLG_V0: &[u8] = &[
        0x00, 0x00, 0x00, 0x20, 0x63, 0x73, 0x6c, 0x67, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x08, 0xff, 0xff, 0xff, 0xf8, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x13, 0x88,
    ];

    // Self generated, no V1 test data
    const ENCODED_CSLG_V1: &[u8] = &[
        0x00, 0x00, 0x00, 0x34, b'c', b's', b'l', b'g', 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x80, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F, 0xFF, 0xFF, 0xFF, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7F,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    ];

    // Self generated, no V1 test data
    const ENCODED_CSLG_V1_VARIANT: &[u8] = &[
        0x00, 0x00, 0x00, 0x34, b'c', b's', b'l', b'g', 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F, 0xFF, 0xFF, 0xFF, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7F,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    ];

    #[test]
    fn test_cslg_v0() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_CSLG_V0);

        let cslg = Cslg::decode(buf).expect("failed to decode cslg");

        assert_eq!(
            cslg,
            Cslg {
                composition_to_dts_shift: 8,
                least_decode_to_display_shift: -8,
                greatest_decode_to_display_delta: 8,
                composition_start_time: 0,
                composition_end_time: 5000
            },
        );

        let mut buf = Vec::new();
        cslg.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_CSLG_V0);
    }

    #[test]
    fn test_cslg_v1_encode() {
        let cslg = Cslg {
            composition_to_dts_shift: 2147483648,
            least_decode_to_display_shift: -2147483649,
            greatest_decode_to_display_delta: 8,
            composition_start_time: i64::MIN,
            composition_end_time: i64::MAX,
        };

        let mut buf = Vec::new();
        cslg.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_CSLG_V1);
    }

    #[test]
    fn test_cslg_v1_decode() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_CSLG_V1);

        let cslg = Cslg::decode(buf).expect("failed to decode cslg");

        assert_eq!(
            cslg,
            Cslg {
                composition_to_dts_shift: 2147483648,
                least_decode_to_display_shift: -2147483649,
                greatest_decode_to_display_delta: 8,
                composition_start_time: i64::MIN,
                composition_end_time: i64::MAX,
            },
        );
    }

    #[test]
    fn test_cslg_v1_encode_variant() {
        let cslg = Cslg {
            composition_to_dts_shift: 2147483647,
            least_decode_to_display_shift: -2147483649,
            greatest_decode_to_display_delta: 8,
            composition_start_time: i64::MIN,
            composition_end_time: i64::MAX,
        };

        let mut buf = Vec::new();
        cslg.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_CSLG_V1_VARIANT);
    }
}
