use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ClockType {
    #[default]
    Unknown = 0,
    DoesNotSync = 1,
    CanSync = 2,
    Reserved = 3,
}

impl From<u8> for ClockType {
    fn from(value: u8) -> Self {
        match value {
            0 => ClockType::Unknown,
            1 => ClockType::DoesNotSync,
            2 => ClockType::CanSync,
            _ => ClockType::Reserved,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Taic {
    pub time_uncertainty: u64,
    pub clock_resolution: u32,
    pub clock_drift_rate: i32,
    pub clock_type: ClockType,
}

impl Taic {
    pub fn new(
        time_uncertainty: u64,
        clock_resolution: u32,
        clock_drift_rate: i32,
        clock_type: ClockType,
    ) -> Result<Self> {
        Ok(Taic {
            time_uncertainty,
            clock_resolution,
            clock_drift_rate,
            clock_type,
        })
    }
}

impl AtomExt for Taic {
    type Ext = ();
    const KIND_EXT: FourCC = FourCC::new(b"taic");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let time_uncertainty = u64::decode(buf)?;
        let clock_resolution = u32::decode(buf)?;
        let clock_drift_rate = i32::decode(buf)?;
        let clock_type_with_reserved = u8::decode(buf)?;
        let clock_type: ClockType = (clock_type_with_reserved >> 6).into();
        Ok(Taic {
            time_uncertainty,
            clock_resolution,
            clock_drift_rate,
            clock_type,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.time_uncertainty.encode(buf)?;
        self.clock_resolution.encode(buf)?;
        self.clock_drift_rate.encode(buf)?;
        let clock_type_with_reserved: u8 = (self.clock_type.clone() as u8) << 6;
        clock_type_with_reserved.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENCODED_TAIC: &[u8] = &[
        0x00, 0x00, 0x00, 0x1d, b't', b'a', b'i', b'c', 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x27, 0x10, 0x7f, 0xff, 0xff, 0xff, 0x80,
    ];

    #[test]
    fn test_taic_decode() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_TAIC);

        let taic = Taic::decode(buf).expect("failed to decode taic");

        assert_eq!(
            taic,
            Taic {
                time_uncertainty: u64::MAX,
                clock_resolution: 10000,
                clock_drift_rate: i32::MAX,
                clock_type: ClockType::CanSync,
            }
        );
    }

    #[test]
    fn test_taic_encode() {
        let taic = Taic {
            time_uncertainty: u64::MAX,
            clock_resolution: 10000,
            clock_drift_rate: i32::MAX,
            clock_type: ClockType::CanSync,
        };

        let mut buf = Vec::new();
        taic.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_TAIC);
    }
}
