use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum EmsgVersion {
    V0 { presentation_time_delta: u32 },
    V1 { presentation_time: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Emsg {
    pub version: EmsgVersion,

    pub timescale: u32,
    pub event_duration: u32,
    pub id: u32,
    pub scheme_id_uri: String,
    pub value: String,
    pub message_data: Vec<u8>,
}

impl Atom for Emsg {
    const KIND: FourCC = FourCC::new(b"emsg");

    fn decode_atom(buf: &mut Buf) -> Result<Self> {
        let version = u8::decode(buf)?;
        buf.u24()?;

        Ok(match version {
            0u8 => Emsg {
                scheme_id_uri: buf.decode()?,
                value: buf.decode()?,
                timescale: buf.decode()?,
                version: EmsgVersion::V0 {
                    presentation_time_delta: buf.decode()?,
                },
                event_duration: buf.decode()?,
                id: buf.decode()?,
                message_data: buf.decode()?,
            },
            1u8 => Emsg {
                timescale: buf.decode()?,
                version: EmsgVersion::V1 {
                    presentation_time: buf.decode()?,
                },
                event_duration: buf.decode()?,
                id: buf.decode()?,
                scheme_id_uri: buf.decode()?,
                value: buf.decode()?,
                message_data: buf.decode()?,
            },
            _ => return Err(Error::UnknownVersion(version)),
        })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        match self.version {
            EmsgVersion::V0 {
                presentation_time_delta,
            } => {
                buf.u8(0)?;
                buf.u24(0)?;
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
                buf.u8(1)?;
                buf.u24(0)?;
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
    use super::*;

    #[test]
    fn test_emsg_version0() {
        let decoded = Emsg {
            version: EmsgVersion::V0 {
                presentation_time_delta: 100,
            },
            timescale: 48000,
            event_duration: 200,
            id: 8,
            scheme_id_uri: String::from("foo"),
            value: String::from("foo"),
            message_data: vec![1, 2, 3],
        };

        let mut buf = BufMut::new();
        decoded.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let output = Emsg::decode(&mut buf).unwrap();

        assert_eq!(decoded, output);
    }

    #[test]
    fn test_emsg_version1() {
        let decoded = Emsg {
            version: EmsgVersion::V1 {
                presentation_time: 50000,
            },
            timescale: 48000,
            event_duration: 200,
            id: 8,
            scheme_id_uri: String::from("foo"),
            value: String::from("foo"),
            message_data: vec![3, 2, 1],
        };

        let mut buf = BufMut::new();
        decoded.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let output = Emsg::decode(&mut buf).unwrap();
        assert_eq!(decoded, output);
    }
}
