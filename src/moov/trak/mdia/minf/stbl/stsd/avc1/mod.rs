mod avcc;

// Incomplete H264 decoder, saved for possible future use
//mod golomb;
//mod pps;
//mod sps;

pub use avcc::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        u32::decode(buf)?; // reserved
        u16::decode(buf)?; // reserved
        let data_reference_index = u16::decode(buf)?;

        u32::decode(buf)?; // pre-defined, reserved
        u64::decode(buf)?; // pre-defined
        u32::decode(buf)?; // pre-defined
        let width = u16::decode(buf)?;
        let height = u16::decode(buf)?;
        let horizresolution = FixedPoint::decode(buf)?;
        let vertresolution = FixedPoint::decode(buf)?;
        u32::decode(buf)?; // reserved
        let frame_count = u16::decode(buf)?;
        let compressor = Compressor::decode(buf)?;
        let depth = u16::decode(buf)?;
        i16::decode(buf)?; // pre-defined

        let mut avcc = None;
        while let Some(atom) = Any::decode_maybe(buf)? {
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

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
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
        (-1i16).encode(buf)?; // pre-defined

        self.avcc.encode(buf)?;

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
                length_size: 4,
                sequence_parameter_sets: vec![vec![
                    0x67, 0x64, 0x00, 0x0D, 0xAC, 0xD9, 0x41, 0x41, 0xFA, 0x10, 0x00, 0x00, 0x03,
                    0x00, 0x10, 0x00, 0x00, 0x03, 0x03, 0x20, 0xF1, 0x42, 0x99, 0x60,
                ]],
                picture_parameter_sets: vec![vec![0x68, 0xEB, 0xE3, 0xCB, 0x22, 0xC0]],
                ..Default::default()
            },
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Avc1::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
