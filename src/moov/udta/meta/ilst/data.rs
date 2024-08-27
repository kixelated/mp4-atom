use std::io::{Read, Seek};

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Data {
    pub data: Vec<u8>,
    pub data_type: DataType,
}

impl Atom for Data {
    const KIND: FourCC = FourCC::new(b"data");

    fn decode_atom(buf: &mut Buf) -> Result<Self> {
        let data_type = DataType::try_from(u32::decode(buf)?)?;

        u32::decode(buf)?; // reserved = 0
        let data = buf.rest().to_vec();

        Ok(Data { data, data_type })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        buf.u32(self.data_type.clone() as u32)?;
        buf.u32(0)?; // reserved = 0
        buf.bytes(&self.data)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data() {
        let expected = Data {
            data_type: DataType::Text,
            data: b"test_data".to_vec(),
        };
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Data::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_data_empty() {
        let expected = Data::default();
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Data::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
