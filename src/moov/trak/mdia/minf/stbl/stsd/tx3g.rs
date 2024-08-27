use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tx3g {
    pub data_reference_index: u16,
    pub display_flags: u32,
    pub horizontal_justification: i8,
    pub vertical_justification: i8,
    pub bg_color_rgba: RgbaColor,
    pub box_record: [i16; 4],
    pub style_record: [u8; 12],
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RgbaColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Default for Tx3g {
    fn default() -> Self {
        Tx3g {
            data_reference_index: 0,
            display_flags: 0,
            horizontal_justification: 1,
            vertical_justification: -1,
            bg_color_rgba: RgbaColor {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 255,
            },
            box_record: [0, 0, 0, 0],
            style_record: [0, 0, 0, 0, 0, 1, 0, 16, 255, 255, 255, 255],
        }
    }
}

impl Atom for Tx3g {
    const KIND: FourCC = FourCC::new(b"tx3g");

    fn decode_atom(buf: &mut Buf) -> Result<Self> {
        u32::decode(buf)?; // reserved
        buf.decode()?; // reserved
        let data_reference_index = buf.decode()?;

        let display_flags = u32::decode(buf)?;
        let horizontal_justification = buf.i8()?()?;
        let vertical_justification = buf.i8()?()?;
        let bg_color_rgba = RgbaColor {
            red: buf.u8()?,
            green: buf.u8()?,
            blue: buf.u8()?,
            alpha: buf.u8()?,
        };
        let box_record: [i16; 4] = [
            i16::decode(buf)?,
            i16::decode(buf)?,
            i16::decode(buf)?,
            i16::decode(buf)?,
        ];
        let style_record = buf.fixed::<12>();

        Ok(Tx3g {
            data_reference_index,
            display_flags,
            horizontal_justification,
            vertical_justification,
            bg_color_rgba,
            box_record,
            style_record,
        })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        buf.u32(0)?; // reserved
        buf.u16(0)?; // reserved
        self.data_reference_index.encode(buf)?;
        self.display_flags.encode(buf)?;
        buf.i8(self.horizontal_justification)?;
        buf.i8(self.vertical_justification)?;
        buf.u8(self.bg_color_rgba.red)?;
        buf.u8(self.bg_color_rgba.green)?;
        buf.u8(self.bg_color_rgba.blue)?;
        buf.u8(self.bg_color_rgba.alpha)?;
        for n in 0..4 {
            buf.i16(self.box_record[n])?;
        }
        for n in 0..12 {
            buf.u8(self.style_record[n])?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx3g() {
        let expected = Tx3g {
            data_reference_index: 1,
            display_flags: 0,
            horizontal_justification: 1,
            vertical_justification: -1,
            bg_color_rgba: RgbaColor {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 255,
            },
            box_record: [0, 0, 0, 0],
            style_record: [0, 0, 0, 0, 0, 1, 0, 16, 255, 255, 255, 255],
        };
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Tx3g::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
