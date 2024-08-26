use std::ffi::CString;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum EmsgVersion {
    V0 { presentation_time_delta: u32 },
    V1 { presentation_time: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Emsg {
    pub version: EmsgVersion,
    pub flags: [u8; 3],
    pub timescale: u32,
    pub event_duration: u32,
    pub id: u32,
    pub scheme_id_uri: String,
    pub value: String,
    pub message_data: Vec<u8>,
}

impl Atom for Emsg {
    const KIND: FourCC = FourCC::new(b"emsg");

    fn encode_inner_size(&self) -> usize {
        self.scheme_id_uri.len()
            + 1 // null terminator
            + self.value.len()
            + 1 // null terminator
            + self.message_data.len()
            + self.timescale.encode_size()
            + self.event_duration.encode_size()
            + self.id.encode_size()
            + match self.version {
                EmsgVersion::V0 { presentation_time_delta } => presentation_time_delta.encode_size(),
                EmsgVersion::V1 { presentation_time } => presentation_time.encode_size(),
            }
    }

    fn decode_inner<B: Buf>(mut buf: &mut B) -> Result<Self> {
        let version = buf.decode()?;
        let flags = buf.decode()?;

        Ok(match version {
            0u8 => {
                let scheme_id_uri = CString::decode(buf)?.into_string()?;
                let value = CString::decode(buf)?.into_string()?;
                let timescale = buf.decode()?;
                let presentation_time_delta = buf.decode()?;
                let event_duration = buf.decode()?;
                let id = buf.decode()?;
                let message_data = buf.decode()?;

                Emsg {
                    version: EmsgVersion::V0 {
                        presentation_time_delta,
                    },
                    flags,
                    timescale,
                    event_duration,
                    id,
                    scheme_id_uri,
                    value,
                    message_data,
                }
            }
            1u8 => {
                let timescale = buf.decode()?;
                let presentation_time = buf.decode()?;
                let event_duration = buf.decode()?;
                let id = buf.decode()?;
                let scheme_id_uri = CString::decode(buf)?.into_string()?;
                let value = CString::decode(buf)?.into_string()?;
                let message_data = buf.decode()?;

                Emsg {
                    version: EmsgVersion::V1 { presentation_time },
                    flags,
                    timescale,
                    event_duration,
                    id,
                    scheme_id_uri,
                    value,
                    message_data,
                }
            }
            _ => return Err(Error::UnknownVersion(version)),
        })
    }

    fn encode_inner<B: BufMut>(&self, buf: &mut B) -> std::result::Result<(), Error> {
        match self.version {
            EmsgVersion::V0 {
                presentation_time_delta,
            } => {
                0u32.encode(buf)?;
                self.flags.encode(buf)?;
                self.scheme_id_uri.as_str().encode(buf)?;
                0u8.encode(buf)?;
                self.value.as_str().encode(buf)?;
                0u8.encode(buf)?;
                self.timescale.encode(buf)?;
                presentation_time_delta.encode(buf)?;
                self.event_duration.encode(buf)?;
                self.id.encode(buf)?;
            }
            EmsgVersion::V1 { presentation_time } => {
                1u32.encode(buf)?;
                self.flags.encode(buf)?;
                self.timescale.encode(buf)?;
                presentation_time.encode(buf)?;
                self.event_duration.encode(buf)?;
                self.id.encode(buf)?;
                self.scheme_id_uri.as_str().encode(buf)?;
                0u8.encode(buf)?;
                self.value.as_str().encode(buf)?;
                0u8.encode(buf)?;
            }
        }

        self.message_data.encode(buf)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_emsg_version0() {
        let decoded = Emsg {
            version: EmsgVersion::V0 {
                presentation_time_delta: 100,
            },
            flags: [0, 0, 0],
            timescale: 48000,
            event_duration: 200,
            id: 8,
            scheme_id_uri: String::from("foo"),
            value: String::from("foo"),
            message_data: vec![1, 2, 3],
        };

        let mut buf = Vec::new();
        decoded.encode(&mut buf).unwrap();

        let mut reader = Cursor::new(&buf);
        let output = Emsg::decode(&mut reader).unwrap();

        assert_eq!(decoded, output);
    }

    #[test]
    fn test_emsg_version1() {
        let decoded = Emsg {
            version: EmsgVersion::V1 {
                presentation_time: 50000,
            },
            flags: [0, 0, 0],
            timescale: 48000,
            event_duration: 200,
            id: 8,
            scheme_id_uri: String::from("foo"),
            value: String::from("foo"),
            message_data: vec![3, 2, 1],
        };

        let mut buf = Vec::new();
        decoded.encode(&mut buf).unwrap();

        let mut reader = Cursor::new(&buf);
        let output = Emsg::decode(&mut reader).unwrap();
        assert_eq!(decoded, output);
    }
}
