use crate::*;

ext! {
    name: Emsg,
    versions: [0, 1],
    flags: {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmsgTimestamp {
    Relative(u32),
    Absolute(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Emsg {
    pub timescale: u32,
    pub presentation_time: EmsgTimestamp,
    pub event_duration: u32,
    pub id: u32,
    pub scheme_id_uri: String,
    pub value: String,
    pub message_data: Bytes,
}

impl AtomExt for Emsg {
    const KIND_EXT: FourCC = FourCC::new(b"emsg");

    type Ext = EmsgExt;

    fn decode_body_ext(buf: &mut Bytes, ext: EmsgExt) -> Result<Self> {
        Ok(match ext.version {
            EmsgVersion::V0 => Emsg {
                scheme_id_uri: buf.decode()?,
                value: buf.decode()?,
                timescale: buf.decode()?,
                presentation_time: EmsgTimestamp::Relative(buf.decode()?),
                event_duration: buf.decode()?,
                id: buf.decode()?,
                message_data: buf.decode()?,
            },
            EmsgVersion::V1 => Emsg {
                timescale: buf.decode()?,
                presentation_time: EmsgTimestamp::Absolute(buf.decode()?),
                event_duration: buf.decode()?,
                id: buf.decode()?,
                scheme_id_uri: buf.decode()?,
                value: buf.decode()?,
                message_data: buf.decode()?,
            },
        })
    }

    fn encode_body_ext(&self, buf: &mut BytesMut) -> Result<EmsgExt> {
        Ok(match self.presentation_time {
            EmsgTimestamp::Absolute(presentation_time) => {
                self.timescale.encode(buf)?;
                presentation_time.encode(buf)?;
                self.event_duration.encode(buf)?;
                self.id.encode(buf)?;
                self.scheme_id_uri.as_str().encode(buf)?;
                self.value.as_str().encode(buf)?;
                self.message_data.encode(buf)?;

                EmsgVersion::V1.into()
            }
            EmsgTimestamp::Relative(presentation_time) => {
                self.scheme_id_uri.as_str().encode(buf)?;
                self.value.as_str().encode(buf)?;
                self.timescale.encode(buf)?;
                presentation_time.encode(buf)?;
                self.event_duration.encode(buf)?;
                self.id.encode(buf)?;
                self.message_data.encode(buf)?;

                EmsgVersion::V0.into()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emsg_version0() {
        let decoded = Emsg {
            timescale: 48000,
            event_duration: 200,
            presentation_time: EmsgTimestamp::Relative(100),
            id: 8,
            scheme_id_uri: String::from("foo"),
            value: String::from("foo"),
            message_data: Bytes::from_static(&[1, 2, 3]),
        };

        let mut buf = BytesMut::new();
        decoded.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let output = buf.decode().unwrap();

        assert_eq!(decoded, output);
    }

    #[test]
    fn test_emsg_version1() {
        let decoded = Emsg {
            presentation_time: EmsgTimestamp::Absolute(50000),
            timescale: 48000,
            event_duration: 200,
            id: 8,
            scheme_id_uri: String::from("foo"),
            value: String::from("foo"),
            message_data: Bytes::from_static(&[3, 2, 1]),
        };

        let mut buf = BytesMut::new();
        decoded.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let output = buf.decode().unwrap();
        assert_eq!(decoded, output);
    }
}
