use crate::*;

// CleanApertureProperty.
// ISO/IEC 23008-12:2022 Section 6.5.9.
// Cropping operation (transformative property).
// Same syntax as CleanApertureBox, ISO/IEC 14496:2022 Section 12.1.4.
// Note the syntax says horizOffN and vertOffN are unsigned. That is wrong. See the definition text and https://github.com/MPEGGroup/FileFormat/issues/41

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Clap {
    pub clean_aperture_width_n: u32,
    pub clean_aperture_width_d: u32,
    pub clean_aperture_height_n: u32,
    pub clean_aperture_height_d: u32,
    pub horiz_off_n: i32,
    pub horiz_off_d: u32,
    pub vert_off_n: i32,
    pub vert_off_d: u32,
}

impl Atom for Clap {
    const KIND: FourCC = FourCC::new(b"clap");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let clean_aperture_width_n = u32::decode(buf)?;
        let clean_aperture_width_d = u32::decode(buf)?;
        let clean_aperture_height_n = u32::decode(buf)?;
        let clean_aperture_height_d = u32::decode(buf)?;
        let horiz_off_n = i32::decode(buf)?;
        let horiz_off_d = u32::decode(buf)?;
        let vert_off_n = i32::decode(buf)?;
        let vert_off_d = u32::decode(buf)?;
        Ok(Clap {
            clean_aperture_width_n,
            clean_aperture_width_d,
            clean_aperture_height_n,
            clean_aperture_height_d,
            horiz_off_n,
            horiz_off_d,
            vert_off_n,
            vert_off_d,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.clean_aperture_width_n.encode(buf)?;
        self.clean_aperture_width_d.encode(buf)?;
        self.clean_aperture_height_n.encode(buf)?;
        self.clean_aperture_height_d.encode(buf)?;
        self.horiz_off_n.encode(buf)?;
        self.horiz_off_d.encode(buf)?;
        self.vert_off_n.encode(buf)?;
        self.vert_off_d.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clap() {
        let expected = Clap {
            clean_aperture_width_n: 100,
            clean_aperture_width_d: 1,
            clean_aperture_height_n: 200,
            clean_aperture_height_d: 3,
            horiz_off_n: 300,
            horiz_off_d: 5,
            vert_off_n: 499,
            vert_off_d: 7,
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Clap::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
