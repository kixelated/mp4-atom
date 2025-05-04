use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ccst {
    pub all_ref_pics_intra: bool,
    pub intra_pred_used: bool,
    pub max_ref_per_pic: u8,
}

impl AtomExt for Ccst {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"ccst");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let bits = u32::decode(buf)?;
        let all_ref_pics_intra = (bits & 0x80000000u32) != 0;
        let intra_pred_used = (bits & 0x40000000u32) != 0;
        let max_ref_per_pic = ((bits & 0x3c000000u32) >> 26) as u8;
        Ok(Ccst {
            all_ref_pics_intra,
            intra_pred_used,
            max_ref_per_pic,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let mut bits = 0u32;
        if self.all_ref_pics_intra {
            bits |= 0x80000000u32;
        }
        if self.intra_pred_used {
            bits |= 0x40000000u32;
        }
        let max_ref_per_pic = (self.max_ref_per_pic as u32) << 26;
        bits |= max_ref_per_pic & 0x3c000000u32;
        bits.encode(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ccst_true_false_1() {
        let expected = Ccst {
            all_ref_pics_intra: true,
            intra_pred_used: false,
            max_ref_per_pic: 1,
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        assert_eq!(
            buf,
            [0, 0, 0, 0x10, b'c', b'c', b's', b't', 0, 0, 0, 0, 0b10000100, 0x00, 0x00, 0x00]
        );
        let decoded = Ccst::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_ccst_false_true_15() {
        let expected = Ccst {
            all_ref_pics_intra: false,
            intra_pred_used: true,
            max_ref_per_pic: 15,
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        assert_eq!(
            buf,
            [0, 0, 0, 0x10, b'c', b'c', b's', b't', 0, 0, 0, 0, 0b01111100, 0x00, 0x00, 0x00]
        );
        let decoded = Ccst::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_ccst_true_true_0() {
        let expected = Ccst {
            all_ref_pics_intra: true,
            intra_pred_used: true,
            max_ref_per_pic: 0,
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        assert_eq!(
            buf,
            [0, 0, 0, 0x10, b'c', b'c', b's', b't', 0, 0, 0, 0, 0b11000000, 0x00, 0x00, 0x00]
        );
        let decoded = Ccst::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_ccst_false_false_7() {
        let expected = Ccst {
            all_ref_pics_intra: false,
            intra_pred_used: false,
            max_ref_per_pic: 7,
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        assert_eq!(
            buf,
            [0, 0, 0, 0x10, b'c', b'c', b's', b't', 0, 0, 0, 0, 0b00011100, 0x00, 0x00, 0x00]
        );
        let decoded = Ccst::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
