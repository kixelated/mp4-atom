use num::Integer;

use crate::*;

/// Compact Sample Size Box (stz2)
///
/// Lists the size of each sample in the track.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Stz2 {
    pub entry_sizes: Vec<u16>,
}

impl AtomExt for Stz2 {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"stz2");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        buf.advance(3); // reserved = 0
        let field_size = u8::decode(buf)?;
        let sample_count = u32::decode(buf)?;
        let mut entry_sizes = Vec::<u16>::with_capacity(std::cmp::min(128, sample_count as usize));

        match field_size {
            4 => {
                // This is to handle the case where there are an odd number entry_sizes, and
                // we don't want to add the last one.
                let mut remaining = 0u16;
                for i in 0..sample_count {
                    if i.is_even() {
                        let entry_pair = u8::decode(buf)? as u16;
                        entry_sizes.push(entry_pair >> 4);
                        remaining = entry_pair & 0x0f;
                    } else {
                        entry_sizes.push(remaining);
                    }
                }
            }
            8 => {
                for _ in 0..sample_count {
                    entry_sizes.push(u8::decode(buf)? as u16);
                }
            }
            16 => {
                for _ in 0..sample_count {
                    entry_sizes.push(u16::decode(buf)?);
                }
            }
            _ => {
                return Err(Error::InvalidSize);
            }
        }

        Ok(Stz2 { entry_sizes })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        // field_size is u8, but treating it as u32 is a convenient way
        // to handle the 3 reserved bytes.
        let mut field_size = 4u32;
        for entry_size in &self.entry_sizes {
            if *entry_size >= 16 {
                // need more than 4 bits
                field_size = 8u32;
            }
            if *entry_size > u8::MAX.into() {
                field_size = 16u32;
                break;
            }
        }
        field_size.encode(buf)?;
        let sample_count: u32 = self
            .entry_sizes
            .len()
            .try_into()
            .map_err(|_| Error::TooLarge(Self::KIND))?;
        sample_count.encode(buf)?;
        match field_size {
            16u32 => {
                for entry_size in &self.entry_sizes {
                    entry_size.encode(buf)?;
                }
            }
            8u32 => {
                for entry_size in &self.entry_sizes {
                    let entry_size_u8: u8 = *entry_size as u8;
                    entry_size_u8.encode(buf)?;
                }
            }
            4u32 => {
                let mut packed_entries =
                    Vec::<u8>::with_capacity(self.entry_sizes.len().div_ceil(2));
                for i in 0..self.entry_sizes.len() {
                    let sample_size = self
                        .entry_sizes
                        .get(i)
                        .expect("there should be a value given we are iterating over the length");
                    if (sample_size & 0b1111) != *sample_size {
                        // There is no way this should be able to happen.
                        return Err(Error::InvalidSize);
                    }
                    let sample_size_u4 = (sample_size & 0b1111) as u8;
                    if i.is_even() {
                        packed_entries.push(sample_size_u4 << 4);
                    } else {
                        let i = packed_entries.get_mut(i / 2).expect(
                            "there should be a value given we are iterating over the length",
                        );
                        *i |= sample_size_u4;
                    }
                }
                for packed_entry in packed_entries {
                    packed_entry.encode(buf)?;
                }
            }
            _ => {
                // There is no way this should be able to happen - only ever assign those 3 values, and those are the only valid values.
                return Err(Error::InvalidSize);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stz2_8() {
        let expected = Stz2 {
            entry_sizes: vec![15, 16, 3],
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        assert_eq!(
            buf,
            vec![
                0x00, 0x00, 0x00, 23, b's', b't', b'z', b'2', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x08, 0x00, 0x00, 0x00, 0x03, 0x0f, 0x10, 0x03
            ]
        );
        let decoded = Stz2::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_stz2_4() {
        let expected = Stz2 {
            entry_sizes: vec![15, 3, 6],
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        assert_eq!(
            buf,
            vec![
                0x00, 0x00, 0x00, 22, b's', b't', b'z', b'2', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x04, 0x00, 0x00, 0x00, 0x03, 0xf3, 0x60
            ]
        );
        let decoded = Stz2::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_stz2_4_even() {
        let expected = Stz2 {
            entry_sizes: vec![15, 3, 6, 8],
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        assert_eq!(
            buf,
            vec![
                0x00, 0x00, 0x00, 22, b's', b't', b'z', b'2', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x04, 0x00, 0x00, 0x00, 0x04, 0xf3, 0x68
            ]
        );
        let decoded = Stz2::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_stz2_16() {
        let expected = Stz2 {
            entry_sizes: vec![255, 256, 65535],
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        assert_eq!(
            buf,
            vec![
                0x00, 0x00, 0x00, 26, b's', b't', b'z', b'2', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x10, 0x00, 0x00, 0x00, 0x03, 0x00, 0xff, 0x01, 0x00, 0xff, 0xff,
            ]
        );
        let decoded = Stz2::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
