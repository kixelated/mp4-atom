use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Avc1 {
    pub data_reference_index: u16,
    pub width: u16,
    pub height: u16,
    pub horizresolution: Ratio<u16>,
    pub vertresolution: Ratio<u16>,
    pub frame_count: u16,
    pub depth: u16,
    pub avcc: Avcc,
}

impl Default for Avc1 {
    fn default() -> Self {
        Avc1 {
            data_reference_index: 0,
            width: 0,
            height: 0,
            horizresolution: 0x48.into(),
            vertresolution: 0x48.into(),
            frame_count: 1,
            depth: 0x0018,
            avcc: Avcc::default(),
        }
    }
}

impl Avc1 {
    pub fn new(config: &AvcConfig) -> Self {
        Avc1 {
            data_reference_index: 1,
            width: config.width,
            height: config.height,
            horizresolution: 0x48.into(),
            vertresolution: 0x48.into(),
            frame_count: 1,
            depth: 0x0018,
            avcc: AvcCBox::new(&config.seq_param_set, &config.pic_param_set),
        }
    }
}

impl Atom for Avc1 {
    const KIND: FourCC = FourCC::new(b"avc1");

    fn decode_atom(buf: &mut Buf) -> Result<Self> {
        u32::decode(buf)?; // reserved
        buf.decode()?; // reserved
        let data_reference_index = buf.decode()?;

        u32::decode(buf)?; // pre-defined, reserved
        u64::decode(buf)?; // pre-defined
        u32::decode(buf)?; // pre-defined
        let width = buf.decode()?;
        let height = buf.decode()?;
        let horizresolution = buf.decode()?;
        let vertresolution = buf.decode()?;
        u32::decode(buf)?; // reserved
        let frame_count = buf.decode()?;
        buf.skip(4)?; // compressorname
        let depth = buf.decode()?;
        i16::decode(buf)?; // pre-defined

        let avcc = Avcc::decode(buf)?;

        Ok(Avc1 {
            data_reference_index,
            width,
            height,
            horizresolution,
            vertresolution,
            frame_count,
            depth,
            avcc,
        })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        buf.u32(0)?; // reserved
        buf.u16(0)?; // reserved
        self.data_reference_index.encode(buf)?;

        buf.u32(0)?; // pre-defined, reserved
        buf.u64(0)?; // pre-defined
        buf.u32(0)?; // pre-defined
        self.width.encode(buf)?;
        self.height.encode(buf)?;
        self.horizresolution.encode(buf)?;
        self.vertresolution.encode(buf)?;
        buf.u32(0)?; // reserved
        self.frame_count.encode(buf)?;
        // skip compressorname
        buf.zero(4)?;
        self.depth.encode(buf)?;
        buf.i32(-1)?; // pre-defined

        self.avcc.encode(buf)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Avcc {
    pub configuration_version: u8,
    pub avc_profile_indication: u8,
    pub profile_compatibility: u8,
    pub avc_level_indication: u8,
    pub length_size_minus_one: u8,
    pub sequence_parameter_sets: Vec<Vec<u8>>,
    pub picture_parameter_sets: Vec<Vec<u8>>,
}

impl Avcc {
    pub fn new(sps: &[u8], pps: &[u8]) -> Self {
        Self {
            configuration_version: 1,
            avc_profile_indication: sps[1],
            profile_compatibility: sps[2],
            avc_level_indication: sps[3],
            length_size_minus_one: 0xff, // length_size = 4
            sequence_parameter_sets: vec![Vec::from(sps)],
            picture_parameter_sets: vec![Vec::from(pps)],
        }
    }
}

impl Atom for Avcc {
    const KIND: FourCC = FourCC::new(b"avcC");

    fn decode_atom(buf: &mut Buf) -> Result<Self> {
        let configuration_version = buf.u8()?;
        let avc_profile_indication = buf.u8()?;
        let profile_compatibility = buf.u8()?;
        let avc_level_indication = buf.u8()?;
        let length_size_minus_one = buf.u8()? & 0x3;
        let num_of_spss = buf.u8()? & 0x1F;
        let mut sequence_parameter_sets = Vec::with_capacity(num_of_spss as usize);
        for _ in 0..num_of_spss {
            let size = buf.u16()? as usize;
            let nal = buf.bytes(size)?;
            sequence_parameter_sets.push(nal);
        }
        let num_of_ppss = buf.u8()?;
        let mut picture_parameter_sets = Vec::with_capacity(num_of_ppss as usize);
        for _ in 0..num_of_ppss {
            let size = buf.u16()? as usize;
            let nal = buf.bytes(size)?;
            picture_parameter_sets.push(nal);
        }

        Ok(Avcc {
            configuration_version,
            avc_profile_indication,
            profile_compatibility,
            avc_level_indication,
            length_size_minus_one,
            sequence_parameter_sets,
            picture_parameter_sets,
        })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        buf.u8(self.configuration_version)?;
        buf.u8(self.avc_profile_indication)?;
        buf.u8(self.profile_compatibility)?;
        buf.u8(self.avc_level_indication)?;
        buf.u8(self.length_size_minus_one | 0xFC)?;

        buf.u8(self.sequence_parameter_sets.len() as u8 | 0xE0)?;
        for sps in self.sequence_parameter_sets {
            buf.u16(sps.len() as u16)?;
            buf.bytes(&sps)?;
        }
        buf.u8(self.picture_parameter_sets.len() as u8)?;
        for pps in self.picture_parameter_sets {
            buf.u16(pps.len() as u16)?;
            buf.bytes(&pps)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avc1() {
        let expected = Avc1 {
            data_reference_index: 1,
            width: 320,
            height: 240,
            horizresolution: 0x48.into(),
            vertresolution: 0x48.into(),
            frame_count: 1,
            depth: 24,
            avcc: Avcc {
                configuration_version: 1,
                avc_profile_indication: 100,
                profile_compatibility: 0,
                avc_level_indication: 13,
                length_size_minus_one: 3,
                sequence_parameter_sets: vec![
                    0x67, 0x64, 0x00, 0x0D, 0xAC, 0xD9, 0x41, 0x41, 0xFA, 0x10, 0x00, 0x00, 0x03,
                    0x00, 0x10, 0x00, 0x00, 0x03, 0x03, 0x20, 0xF1, 0x42, 0x99, 0x60,
                ],
                picture_parameter_sets: vec![0x68, 0xEB, 0xE3, 0xCB, 0x22, 0xC0],
            },
        };
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Avc1::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
