use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Avcc {
    pub configuration_version: u8,
    pub avc_profile_indication: u8,
    pub profile_compatibility: u8,
    pub avc_level_indication: u8,
    pub length_size: u8,
    pub sequence_parameter_sets: Vec<Vec<u8>>,
    pub picture_parameter_sets: Vec<Vec<u8>>,
    pub ext: Option<AvccExt>,
}

// Only valid for certain profiles
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AvccExt {
    pub chroma_format: u8,
    pub bit_depth_luma: u8,
    pub bit_depth_chroma: u8,
    pub sequence_parameter_sets_ext: Vec<Vec<u8>>,
}

impl Default for AvccExt {
    fn default() -> Self {
        AvccExt {
            chroma_format: 1,
            bit_depth_luma: 8,
            bit_depth_chroma: 8,
            sequence_parameter_sets_ext: Vec::new(),
        }
    }
}

impl Avcc {
    pub fn new(sps: &[u8], pps: &[u8]) -> Result<Self> {
        if sps.len() < 4 {
            return Err(Error::OutOfBounds);
        }

        Ok(Self {
            configuration_version: 1,
            avc_profile_indication: sps[1],
            profile_compatibility: sps[2],
            avc_level_indication: sps[3],
            length_size: 4,
            sequence_parameter_sets: vec![sps.into()],
            picture_parameter_sets: vec![pps.into()],

            // TODO This information could be parsed out of the SPS
            ext: None,
        })
    }
}

impl Atom for Avcc {
    const KIND: FourCC = FourCC::new(b"avcC");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let configuration_version = u8::decode(buf)?;
        if configuration_version != 1 {
            return Err(Error::UnknownVersion(configuration_version));
        }
        let avc_profile_indication = u8::decode(buf)?;
        let profile_compatibility = u8::decode(buf)?;
        let avc_level_indication = u8::decode(buf)?;

        // The first 6 bits are reserved and the value is encoded -1
        let mut length_size = u8::decode(buf)?;
        length_size = (length_size & 0x03) + 1;

        let num_of_spss = u8::decode(buf)? & 0x1F;
        let mut sequence_parameter_sets = Vec::with_capacity(num_of_spss as usize);
        for _ in 0..num_of_spss {
            let size = u16::decode(buf)? as usize;
            let nal = Vec::decode_exact(buf, size)?;
            sequence_parameter_sets.push(nal);
        }

        let num_of_ppss = u8::decode(buf)?;
        let mut picture_parameter_sets = Vec::with_capacity(num_of_ppss as usize);
        for _ in 0..num_of_ppss {
            let size = u16::decode(buf)? as usize;
            let nal = Vec::decode_exact(buf, size)?;
            picture_parameter_sets.push(nal);
        }

        // NOTE: Many encoders/decoders skip this extended avcC part.
        // It's profile specific, but we don't really care and will parse it if present.
        let ext = if buf.has_remaining() {
            let chroma_format = u8::decode(buf)? & 0x3;
            let bit_depth_luma_minus8 = u8::decode(buf)? & 0x7;
            let bit_depth_chroma_minus8 = u8::decode(buf)? & 0x7;
            let num_of_sequence_parameter_set_exts = u8::decode(buf)? as usize;
            let mut sequence_parameter_sets_ext =
                Vec::with_capacity(num_of_sequence_parameter_set_exts);

            for _ in 0..num_of_sequence_parameter_set_exts {
                let size = u16::decode(buf)? as usize;
                let nal = Vec::decode_exact(buf, size)?;
                sequence_parameter_sets_ext.push(nal);
            }

            Some(AvccExt {
                chroma_format,
                bit_depth_luma: bit_depth_luma_minus8 + 8,
                bit_depth_chroma: bit_depth_chroma_minus8 + 8,
                sequence_parameter_sets_ext,
            })
        } else {
            None
        };

        Ok(Avcc {
            configuration_version,
            avc_profile_indication,
            profile_compatibility,
            avc_level_indication,
            length_size,
            sequence_parameter_sets,
            picture_parameter_sets,
            ext,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.configuration_version.encode(buf)?;
        self.avc_profile_indication.encode(buf)?;
        self.profile_compatibility.encode(buf)?;
        self.avc_level_indication.encode(buf)?;
        let length_size = match self.length_size {
            0 => return Err(Error::InvalidSize),
            1..=4 => self.length_size - 1,
            _ => return Err(Error::InvalidSize),
        };
        (length_size | 0xFC).encode(buf)?;

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

        if let Some(ext) = &self.ext {
            ok_in_range(ext.chroma_format, 0..4)
                .map(|n| n | 0b11111100)?
                .encode(buf)?;
            ok_in_range(
                ext.bit_depth_luma
                    .checked_sub(8)
                    .ok_or(Error::InvalidSize)?,
                0..8,
            )
            .map(|n| n | 0b11111000)?
            .encode(buf)?;
            ok_in_range(
                ext.bit_depth_chroma
                    .checked_sub(8)
                    .ok_or(Error::InvalidSize)?,
                0..8,
            )
            .map(|n| n | 0b11111000)?
            .encode(buf)?;
            (ext.sequence_parameter_sets_ext.len() as u8).encode(buf)?;
            for sps in &ext.sequence_parameter_sets_ext {
                (sps.len() as u16).encode(buf)?;
                sps.encode(buf)?;
            }
        }

        Ok(())
    }
}

/// Returns the input `n` if it is within the provided `range`. Otherwise, it returns an
/// [`Error::InvalidSize`] error.
fn ok_in_range(n: u8, range: std::ops::Range<u8>) -> Result<u8> {
    if range.contains(&n) {
        Ok(n)
    } else {
        Err(Error::InvalidSize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn ok_in_range_is_ok_when_in_range() {
        assert_eq!(1, ok_in_range(1, 0..8).expect("should be Ok"));
    }

    #[test]
    fn ok_in_range_is_err_when_out_of_range() {
        match ok_in_range(8, 0..8) {
            Err(Error::InvalidSize) => (),
            Ok(n) => panic!("unexpected Ok value Ok({n})"),
            Err(e) => panic!("unexpected error case {e}"),
        }
    }

    // This example was taken from:
    // https://devstreaming-cdn.apple.com/videos/streaming/examples/img_bipbop_adv_example_fmp4/v8/main.mp4
    //
    // I just extracted the bytes for the avcc atom location.
    //
    // NOTE: this atom is badly configured, in that the reserved bits before lengthSizeMinusOne are
    // supposed to be 0b111111 and the reserved bits before numOfSequenceParameterSets are supposed
    // to be 0b111; however, in this example, they are 0b000000 and 0b000. The test therefore also
    // validates relaxed validation in decoding.
    const ENCODED: &[u8] = &[
        0x00, 0x00, 0x00, 0x41, 0x61, 0x76, 0x63, 0x43, 0x01, 0x64, 0x00, 0x2A, 0x03, 0x01, 0x00,
        0x26, 0x27, 0x64, 0x00, 0x2A, 0xAC, 0x24, 0x8C, 0x07, 0x80, 0x22, 0x7E, 0x5C, 0x04, 0x40,
        0x00, 0x00, 0x03, 0x00, 0x40, 0x00, 0x00, 0x1E, 0x38, 0xA0, 0x00, 0x0B, 0x71, 0xB0, 0x00,
        0x16, 0xE3, 0x7B, 0xDE, 0xE0, 0x3E, 0x11, 0x08, 0xA7, 0x01, 0x00, 0x04, 0x28, 0xDE, 0xBC,
        0xB0, 0xFD, 0xF8, 0xF8, 0x00,
    ];

    #[test]
    fn avcc_decodes_correctly() {
        let mut buf = Cursor::new(ENCODED);
        let avcc = Avcc {
            configuration_version: 1,
            avc_profile_indication: 100,
            profile_compatibility: 0,
            avc_level_indication: 42,
            length_size: 4,
            sequence_parameter_sets: vec![ENCODED[16..54].to_vec()],
            picture_parameter_sets: vec![ENCODED[57..61].to_vec()],
            ext: Some(AvccExt {
                chroma_format: 1,
                bit_depth_luma: 8,
                bit_depth_chroma: 8,
                sequence_parameter_sets_ext: Vec::new(),
            }),
        };
        let decoded = Avcc::decode(&mut buf).expect("avcC should decode successfully");
        assert_eq!(avcc, decoded);
        let mut encoded = Vec::new();
        avcc.encode(&mut encoded)
            .expect("encode should be successful");
        // Here we fix the encoded bytes so that the problem reserved bits are set properly to
        // 0b111111 and 0b111.
        let mut fixed_encoded = ENCODED.to_vec();
        if let Some(length_size_minus_one) = fixed_encoded.get_mut(12) {
            *length_size_minus_one += 0b11111100;
        }
        if let Some(num_of_sequence_parameter_sets) = fixed_encoded.get_mut(13) {
            *num_of_sequence_parameter_sets += 0b11100000;
        }
        assert_eq!(fixed_encoded, encoded);
    }
}
