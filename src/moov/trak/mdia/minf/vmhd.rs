use crate::*;

#[derive(Default)]
pub(crate) struct VmhdExt;

impl Ext for VmhdExt {
    fn encode(&self) -> Result<u32> {
        // ISO/IEC 14496-12 requires vmhd to use version 0 and flags 1.
        Ok(1)
    }

    fn decode(_: u32) -> Result<Self> {
        Ok(Self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vmhd {
    pub graphics_mode: u16,
    pub op_color: RgbColor,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RgbColor {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
}

impl AtomExt for Vmhd {
    type Ext = VmhdExt;

    const KIND_EXT: FourCC = FourCC::new(b"vmhd");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: VmhdExt) -> Result<Self> {
        let graphics_mode = u16::decode(buf)?;
        let op_color = RgbColor {
            red: u16::decode(buf)?,
            green: u16::decode(buf)?,
            blue: u16::decode(buf)?,
        };

        Ok(Vmhd {
            graphics_mode,
            op_color,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<VmhdExt> {
        self.graphics_mode.encode(buf)?;
        self.op_color.red.encode(buf)?;
        self.op_color.green.encode(buf)?;
        self.op_color.blue.encode(buf)?;

        Ok(VmhdExt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENCODED_VMHD: &[u8] = &[
        0x00, 0x00, 0x00, 0x14, b'v', b'm', b'h', b'd', 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn test_vmhd_encode() {
        let expected = Vmhd {
            graphics_mode: 0,
            op_color: RgbColor {
                red: 0,
                green: 0,
                blue: 0,
            },
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_VMHD);
    }

    #[test]
    fn test_vmhd_decode() {
        let mut buf = std::io::Cursor::new(ENCODED_VMHD);
        let decoded = Vmhd::decode(&mut buf).unwrap();

        let expected = Vmhd {
            graphics_mode: 0,
            op_color: RgbColor {
                red: 0,
                green: 0,
                blue: 0,
            },
        };
        assert_eq!(decoded, expected);
    }
}
