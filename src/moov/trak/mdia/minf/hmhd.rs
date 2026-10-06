use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hmhd {
    pub max_pdu_size: u16,
    pub avg_pdu_size: u16,
    pub max_bitrate: u32,
    pub avg_bitrate: u32,
}

impl AtomExt for Hmhd {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"hmhd");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let max_pdu_size = u16::decode(buf)?;
        let avg_pdu_size = u16::decode(buf)?;
        let max_bitrate = u32::decode(buf)?;
        let avg_bitrate = u32::decode(buf)?;
        u32::decode(buf)?; // reserved
        Ok(Hmhd {
            max_pdu_size,
            avg_pdu_size,
            max_bitrate,
            avg_bitrate,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.max_pdu_size.encode(buf)?;
        self.avg_pdu_size.encode(buf)?;
        self.max_bitrate.encode(buf)?;
        self.avg_bitrate.encode(buf)?;
        0u32.encode(buf)?; // reserved
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmhd() {
        let hmhd = Hmhd {
            max_pdu_size: 400,
            avg_pdu_size: 115,
            max_bitrate: 6700,
            avg_bitrate: 4123,
        };
        let mut buf = Vec::new();
        hmhd.encode(&mut buf).unwrap();
        assert_eq!(
            buf,
            vec![
                0x00, 0x00, 0x00, 0x1c, 0x68, 0x6d, 0x68, 0x64, 0x00, 0x00, 0x00, 0x00, 0x01, 0x90,
                0x00, 0x73, 0x00, 0x00, 0x1a, 0x2c, 0x00, 0x00, 0x10, 0x1b, 0x00, 0x00, 0x00, 0x00
            ]
        );

        let mut buf = buf.as_ref();
        let decoded = Hmhd::decode(&mut buf).unwrap();
        assert_eq!(decoded, hmhd);
    }
}
