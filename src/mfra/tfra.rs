use crate::*;

ext! {
    name: Tfra,
    versions: [0, 1],
    flags: {}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FragmentInfo {
    pub time: u64,
    pub moof_offset: u64,
    pub traf_number: u32,
    pub trun_number: u32,
    pub sample_delta: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tfra {
    pub track_id: u32,
    pub entries: Vec<FragmentInfo>,
}
impl Tfra {
    fn determine_required_lengths(&self) -> (u32, u32, u32, TfraVersion) {
        let mut length_size_of_traf_num = 0;
        let mut length_size_of_trun_num = 0;
        let mut length_size_of_sample_num = 0;
        let mut version = TfraVersion::V0;
        for entry in &self.entries {
            if entry.time > u32::MAX.into() || entry.moof_offset > u32::MAX.into() {
                version = TfraVersion::V1;
            }
            length_size_of_traf_num = std::cmp::max(
                length_size_of_traf_num,
                determine_required_length(entry.traf_number),
            );
            length_size_of_trun_num = std::cmp::max(
                length_size_of_trun_num,
                determine_required_length(entry.trun_number),
            );
            length_size_of_sample_num = std::cmp::max(
                length_size_of_sample_num,
                determine_required_length(entry.sample_delta),
            );
        }
        (
            length_size_of_traf_num,
            length_size_of_trun_num,
            length_size_of_sample_num,
            version,
        )
    }
}

fn determine_required_length(value: u32) -> u32 {
    // number of bytes required minus 1
    if value > u24::MAX {
        3
    } else if value > u16::MAX.into() {
        2
    } else if value > u8::MAX.into() {
        1
    } else {
        0
    }
}

fn decode_variable_unsigned_int<B: Buf>(buf: &mut B, num_bits_minus_one: u32) -> Result<u32> {
    match num_bits_minus_one {
        0 => Ok(u8::decode(buf)? as u32),
        1 => Ok(u16::decode(buf)? as u32),
        2 => Ok(u24::decode(buf)?.into()),
        3 => u32::decode(buf),
        _ => Err(Error::InvalidSize),
    }
}

fn encode_variable_unsigned_int<B: BufMut>(
    buf: &mut B,
    value: u32,
    num_bits_minus_one: u32,
) -> Result<()> {
    match num_bits_minus_one {
        0 => (value as u8).encode(buf),
        1 => (value as u16).encode(buf),
        2 => {
            let v: u24 = value.try_into().expect("should have already been checked");
            v.encode(buf)
        }
        3 => value.encode(buf),
        _ => Err(Error::InvalidSize),
    }
}

impl AtomExt for Tfra {
    const KIND_EXT: FourCC = FourCC::new(b"tfra");

    type Ext = TfraExt;

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: TfraExt) -> Result<Self> {
        let track_id = u32::decode(buf)?;
        let lengths = u32::decode(buf)?;
        // high 26 bits are reserved, low 6 bits indicate length used later
        let length_size_of_sample_num = lengths & 0b11;
        let length_size_of_trun_num = (lengths >> 2) & 0b11;
        let length_size_of_traf_num = (lengths >> 4) & 0b11;
        let number_of_entry = u32::decode(buf)?;
        // Don't trust the entry count, just start with a small-ish reservation
        let mut entries = Vec::with_capacity(std::cmp::min(128, number_of_entry as usize));
        for _ in 0..number_of_entry {
            let (time, moof_offset) = match ext.version {
                TfraVersion::V1 => (u64::decode(buf)?, u64::decode(buf)?),
                TfraVersion::V0 => (u32::decode(buf)?.into(), u32::decode(buf)?.into()),
            };
            let fragment_info = FragmentInfo {
                time,
                moof_offset,
                traf_number: decode_variable_unsigned_int(buf, length_size_of_traf_num)?,
                trun_number: decode_variable_unsigned_int(buf, length_size_of_trun_num)?,
                sample_delta: decode_variable_unsigned_int(buf, length_size_of_sample_num)?,
            };
            entries.push(fragment_info);
        }
        Ok(Tfra { track_id, entries })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<TfraExt> {
        self.track_id.encode(buf)?;
        let (length_size_of_traf_num, length_size_of_trun_num, length_size_of_sample_num, version) =
            self.determine_required_lengths();
        ((length_size_of_traf_num << 4)
            | (length_size_of_trun_num << 2)
            | (length_size_of_sample_num))
            .encode(buf)?;
        let number_of_entry: u32 = self
            .entries
            .len()
            .try_into()
            .map_err(|_| Error::TooLarge(Self::KIND))?;
        number_of_entry.encode(buf)?;
        for entry in &self.entries {
            match version {
                TfraVersion::V1 => {
                    entry.time.encode(buf)?;
                    entry.moof_offset.encode(buf)?
                }
                TfraVersion::V0 => {
                    (entry.time as u32).encode(buf)?;
                    (entry.moof_offset as u32).encode(buf)?;
                }
            }
            encode_variable_unsigned_int(buf, entry.traf_number, length_size_of_traf_num)?;
            encode_variable_unsigned_int(buf, entry.trun_number, length_size_of_trun_num)?;
            encode_variable_unsigned_int(buf, entry.sample_delta, length_size_of_sample_num)?;
        }
        Ok(version.into())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    // We only test V1 here, V0 gets checked in the parent mod test

    // From MPEG File Format Conformance suite: uvvu/Solekai007_1920_29_1x1_v7clear.uvu
    // with a change to make it actually require V1
    const ENCODED_TFRA: &[u8] = &[
        0x00, 0x00, 0x00, 0x2c, 0x74, 0x66, 0x72, 0x61, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x09, 0x68, 0x45, 0x01, 0x02, 0xFF, 0xFF,
    ];

    #[test]
    fn test_tfra_v1_decode() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_TFRA);
        let tfra = Tfra::decode(buf).expect("failed to decode tfra");
        assert_eq!(
            tfra,
            Tfra {
                track_id: 3,
                entries: vec![FragmentInfo {
                    time: 0,
                    moof_offset: 1099512244293,
                    traf_number: 1,
                    trun_number: 2,
                    sample_delta: 65535
                }]
            }
        );
    }

    #[test]
    fn test_tfra_v1_encode() {
        let tfra = Tfra {
            track_id: 3,
            entries: vec![FragmentInfo {
                time: 0,
                moof_offset: 1099512244293,
                traf_number: 1,
                trun_number: 2,
                sample_delta: 65535,
            }],
        };

        let mut buf = Vec::new();
        tfra.encode(&mut buf).unwrap();
        assert_eq!(buf, ENCODED_TFRA);
    }
}
