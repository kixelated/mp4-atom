use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Vmhd {
    pub graphics_mode: u16,
    pub op_color: RgbColor,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RgbColor {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
}

impl AtomExt for Vmhd {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"vmhd");

    fn decode_body_ext(buf: &mut Bytes, _ext: ()) -> Result<Self> {
        let graphics_mode = buf.decode()?;
        let op_color = RgbColor {
            red: buf.decode()?,
            green: buf.decode()?,
            blue: buf.decode()?,
        };

        Ok(Vmhd {
            graphics_mode,
            op_color,
        })
    }

    fn encode_body_ext(&self, buf: &mut BytesMut) -> Result<()> {
        self.graphics_mode.encode(buf)?;
        self.op_color.red.encode(buf)?;
        self.op_color.green.encode(buf)?;
        self.op_color.blue.encode(buf)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vmhd() {
        let expected = Vmhd {
            graphics_mode: 0,
            op_color: RgbColor {
                red: 0,
                green: 0,
                blue: 0,
            },
        };
        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let decoded = Vmhd::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
