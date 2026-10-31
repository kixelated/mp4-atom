use crate::*;

/// Text track sample description format for tx3g
///
/// 3GPP TS 26.245 or ETSI TS 126 245 Section 5.16
/// See https://www.etsi.org/deliver/etsi_ts/126200_126299/126245/18.00.00_60/ts_126245v180000p.pdf
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tx3g {
    pub data_reference_index: u16,
    pub display_flags: u32,
    pub horizontal_justification: i8,
    pub vertical_justification: i8,
    pub bg_color_rgba: RgbaColor,
    pub box_record: [i16; 4],
    pub style_record: [u8; 12],
    // Looks like this is supposed to be present, but we're relaxed about it.
    pub ftab: Option<Ftab>,
    pub disp: Option<Disp>,
}

/// `DisparityBox` ('disp'): the stereo disparity shift of timed text. In a
/// sample entry it is the default, in a sample it overrides the default.
///
/// See 3GPP TS 26.245 Section 5.17.1.9.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Disp {
    /// 16 times the shift applied to the left on the left view and to the
    /// right on the right view.
    pub disparity_shift_in_16th_pel: i16,
}

impl Atom for Disp {
    const KIND: FourCC = FourCC::new(b"disp");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Self {
            disparity_shift_in_16th_pel: i16::decode(buf)?,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.disparity_shift_in_16th_pel.encode(buf)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RgbaColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Decode for RgbaColor {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Self {
            red: u8::decode(buf)?,
            green: u8::decode(buf)?,
            blue: u8::decode(buf)?,
            alpha: u8::decode(buf)?,
        })
    }
}

impl Encode for RgbaColor {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.red.encode(buf)?;
        self.green.encode(buf)?;
        self.blue.encode(buf)?;
        self.alpha.encode(buf)
    }
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
            ftab: None, // TODO: possibly a nice Serif?
            disp: None,
        }
    }
}

impl Atom for Tx3g {
    const KIND: FourCC = FourCC::new(b"tx3g");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        u32::decode(buf)?; // reserved
        u16::decode(buf)?; // reserved
        let data_reference_index = u16::decode(buf)?;

        let display_flags = u32::decode(buf)?;
        let horizontal_justification = i8::decode(buf)?;
        let vertical_justification = i8::decode(buf)?;
        let bg_color_rgba = RgbaColor::decode(buf)?;
        let box_record: [i16; 4] = [
            i16::decode(buf)?,
            i16::decode(buf)?,
            i16::decode(buf)?,
            i16::decode(buf)?,
        ];
        let style_record = <[u8; 12]>::decode(buf)?;
        let mut ftab = None;
        let mut disp = None;
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Ftab(atom) => ftab.insert_if_none(atom)?,
                Any::Disp(atom) => disp.insert_if_none(atom)?,
                unknown => Self::decode_unknown(&unknown)?,
            }
        }
        skip_trailing_padding(buf);

        Ok(Tx3g {
            data_reference_index,
            display_flags,
            horizontal_justification,
            vertical_justification,
            bg_color_rgba,
            box_record,
            style_record,
            ftab,
            disp,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        0u32.encode(buf)?; // reserved
        0u16.encode(buf)?; // reserved
        self.data_reference_index.encode(buf)?;
        self.display_flags.encode(buf)?;
        self.horizontal_justification.encode(buf)?;
        self.vertical_justification.encode(buf)?;
        self.bg_color_rgba.encode(buf)?;
        for n in 0..4 {
            (self.box_record[n]).encode(buf)?;
        }
        for n in 0..12 {
            (self.style_record[n]).encode(buf)?;
        }
        self.ftab.encode(buf)?;
        self.disp.encode(buf)?;

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
            ftab: None,
            disp: None,
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Tx3g::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    // From the MPEG file format conformance test suite: isobmff/09_text.mp4
    const ENCODED_TX3G_09: &[u8] = &[
        0x00, 0x00, 0x00, 0x40, 0x74, 0x78, 0x33, 0x67, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x04, 0x00, 0x00, 0x01, 0xff, 0xff, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x3c, 0x01, 0x90, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x12, 0xff, 0xff, 0xff,
        0xff, 0x00, 0x00, 0x00, 0x12, 0x66, 0x74, 0x61, 0x62, 0x00, 0x01, 0x00, 0x01, 0x05, 0x53,
        0x65, 0x72, 0x69, 0x66,
    ];

    /* From isobmff/09_text_gpac.json:
           "@Size": "64",
           "@Type": "tx3g",
           "@Specification": "3gpp",
           "@Container": "stsd",
           "@dataReferenceIndex": "1",
           "@displayFlags": "40000",
           "@horizontal-justification": "1",
           "@vertical-justification": "-1",
           "@backgroundColor": "ff 0 0 ff",
           "DefaultBox": {
               "BoxRecord": {
               "@top": "0",
               "@left": "0",
               "@bottom": "60",
               "@right": "400"
               },
               "FontTableBox": {
               "@Size": "18",
               "@Type": "ftab",
               "@Specification": "3gpp",
               "@Container": "tx3g text enct",
               "FontRecord": {
                   "@ID": "1",
                   "@name": "Serif"
               }
               }
           },
           "DefaultStyle": {
               "StyleRecord": {
               "@startChar": "0",
               "@endChar": "0",
               "@fontID": "1",
               "@styles": "Normal",
               "@fontSize": "18",
               "@textColor": "ff ff ff ff"
               }
           },
           "FontTableBox": {
               "@Size": "18",
               "@Type": "ftab",
               "@Specification": "3gpp",
               "@Container": "tx3g text enct",
               "FontRecord": {
               "@ID": "1",
               "@name": "Serif"
               }
           }
           },
    */

    fn get_reference_mpeg_09_tx3g() -> Tx3g {
        Tx3g {
            data_reference_index: 1,
            display_flags: 0x00040000,
            horizontal_justification: 1,
            vertical_justification: -1,
            bg_color_rgba: RgbaColor {
                red: 0xFF,
                green: 0x00,
                blue: 0x00,
                alpha: 0xFF,
            },
            box_record: [0, 0, 60, 400],
            style_record: [0, 0, 0, 0, 0, 1, 0, 18, 255, 255, 255, 255],
            ftab: Some(Ftab {
                font_entries: vec![FontEntry {
                    font_id: 1,
                    font: "Serif".into(),
                }],
            }),
            disp: None,
        }
    }

    #[test]
    fn test_tx3g_with_default_disparity() {
        let expected = Tx3g {
            disp: Some(Disp {
                disparity_shift_in_16th_pel: 48,
            }),
            ..get_reference_mpeg_09_tx3g()
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();
        let decoded = Tx3g::decode(&mut buf.as_slice()).expect("failed to decode tx3g");
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_mpeg_09_text_decode() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_TX3G_09);
        let tx3g = Tx3g::decode(buf).expect("failed to decode tx3g");
        assert_eq!(tx3g, get_reference_mpeg_09_tx3g());
    }

    #[test]
    fn test_mpeg_09_text_encode() {
        let tx3g = get_reference_mpeg_09_tx3g();
        let mut buf = Vec::new();
        tx3g.encode(&mut buf).unwrap();
        assert_eq!(buf.as_slice(), ENCODED_TX3G_09);
    }
}
