use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Avc1 {
    pub data_reference_index: u16,
    pub width: u16,
    pub height: u16,
    pub horizresolution: FixedPoint<u16>,
    pub vertresolution: FixedPoint<u16>,
    pub frame_count: u16,
    pub compressor: Compressor,
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
            compressor: Default::default(),
            depth: 0x0018,
            avcc: Avcc::default(),
        }
    }
}

impl Atom for Avc1 {
    const KIND: FourCC = FourCC::new(b"avc1");

    fn decode_atom(buf: &mut Bytes) -> Result<Self> {
        u32::decode(buf)?; // reserved
        u16::decode(buf)?; // reserved
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
        let compressor = buf.decode()?;
        let depth = buf.decode()?;
        i16::decode(buf)?; // pre-defined

        let mut avcc = None;
        while let Some(atom) = buf.decode()? {
            match atom {
                Any::Avcc(atom) => avcc = atom.into(),
                _ => tracing::warn!("unknown atom: {:?}", atom),
            }
        }

        Ok(Avc1 {
            data_reference_index,
            width,
            height,
            horizresolution,
            vertresolution,
            frame_count,
            compressor,
            depth,
            avcc: avcc.ok_or(Error::MissingBox(Avcc::KIND))?,
        })
    }

    fn encode_atom(&self, buf: &mut BytesMut) -> Result<()> {
        0u32.encode(buf)?; // reserved
        0u16.encode(buf)?; // reserved
        self.data_reference_index.encode(buf)?;

        0u32.encode(buf)?; // pre-defined, reserved
        0u64.encode(buf)?; // pre-defined
        0u32.encode(buf)?; // pre-defined
        self.width.encode(buf)?;
        self.height.encode(buf)?;
        self.horizresolution.encode(buf)?;
        self.vertresolution.encode(buf)?;
        0u32.encode(buf)?; // reserved
        self.frame_count.encode(buf)?;
        self.compressor.encode(buf)?;
        self.depth.encode(buf)?;
        (-1i32).encode(buf)?; // pre-defined

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
    pub sequence_parameter_sets: Vec<Bytes>,
    pub picture_parameter_sets: Vec<Bytes>,
}

impl Avcc {
    pub fn new(sps: &[u8], pps: &[u8]) -> Self {
        Self {
            configuration_version: 1,
            avc_profile_indication: sps[1],
            profile_compatibility: sps[2],
            avc_level_indication: sps[3],
            length_size_minus_one: 0xff, // length_size = 4
            sequence_parameter_sets: vec![Bytes::copy_from_slice(sps)],
            picture_parameter_sets: vec![Bytes::copy_from_slice(pps)],
        }
    }
}

impl Atom for Avcc {
    const KIND: FourCC = FourCC::new(b"avcC");

    fn decode_atom(buf: &mut Bytes) -> Result<Self> {
        let configuration_version = u8::decode(buf)?;
        let avc_profile_indication = u8::decode(buf)?;
        let profile_compatibility = u8::decode(buf)?;
        let avc_level_indication = u8::decode(buf)?;
        let length_size_minus_one = u8::decode(buf)? & 0x3;
        let num_of_spss = u8::decode(buf)? & 0x1F;

        let mut sequence_parameter_sets = Vec::with_capacity(num_of_spss as usize);
        for _ in 0..num_of_spss {
            let size = u16::decode(buf)? as usize;
            if buf.len() < size {
                return Err(Error::OutOfBounds);
            }
            let nal = buf.split_to(size);
            sequence_parameter_sets.push(nal);
        }

        let num_of_ppss = u8::decode(buf)?;
        let mut picture_parameter_sets = Vec::with_capacity(num_of_ppss as usize);
        for _ in 0..num_of_ppss {
            let size = u16::decode(buf)? as usize;
            if buf.len() < size {
                return Err(Error::OutOfBounds);
            }
            let nal = buf.split_to(size);
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

    fn encode_atom(&self, buf: &mut BytesMut) -> Result<()> {
        self.configuration_version.encode(buf)?;
        self.avc_profile_indication.encode(buf)?;
        self.profile_compatibility.encode(buf)?;
        self.avc_level_indication.encode(buf)?;
        (self.length_size_minus_one | 0xFC).encode(buf)?;
        (self.sequence_parameter_sets.len() as u8 | 0xE0).encode(buf)?;
        for sps in &self.sequence_parameter_sets {
            (sps.len() as u16).encode(buf)?;
            sps.encode(buf)?;
        }
        (self.picture_parameter_sets.len() as u8).encode(buf)?;
        for pps in &self.picture_parameter_sets {
            (pps.len() as u16).encode(buf)?;
            pps.encode(buf)?;
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
            compressor: "ya boy".into(),
            depth: 24,
            avcc: Avcc {
                configuration_version: 1,
                avc_profile_indication: 100,
                profile_compatibility: 0,
                avc_level_indication: 13,
                length_size_minus_one: 3,
                sequence_parameter_sets: vec![Bytes::from_static(&[
                    0x67, 0x64, 0x00, 0x0D, 0xAC, 0xD9, 0x41, 0x41, 0xFA, 0x10, 0x00, 0x00, 0x03,
                    0x00, 0x10, 0x00, 0x00, 0x03, 0x03, 0x20, 0xF1, 0x42, 0x99, 0x60,
                ])],
                picture_parameter_sets: vec![Bytes::from_static(&[
                    0x68, 0xEB, 0xE3, 0xCB, 0x22, 0xC0,
                ])],
            },
        };
        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let decoded = Avc1::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
