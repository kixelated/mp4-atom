use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Esds {
    pub es_desc: EsDescriptor,
}

impl AtomExt for Esds {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"esds");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let mut es_desc = None;

        while let Some(desc) = Descriptor::decode_maybe(buf)? {
            match desc {
                Descriptor::EsDescriptor(desc) => es_desc = Some(desc),
                Descriptor::Unknown(tag, _) => {
                    tracing::warn!("unknown descriptor: {:02X}", tag)
                }
                _ => return Err(Error::UnexpectedDescriptor(desc.tag())),
            }
        }

        Ok(Esds {
            es_desc: es_desc.ok_or(Error::MissingDescriptor(EsDescriptor::TAG))?,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        Descriptor::from(self.es_desc.clone()).encode(buf)
    }
}

macro_rules! descriptors {
    ($($name:ident,)*) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum Descriptor {
            $(
                $name($name),
            )*
            Unknown(u8, Vec<u8>),
        }

        impl Decode for Descriptor {
            fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
                let tag = u8::decode(buf)?;

                let mut size: u32 = 0;
                for index in 0..4 {
                    let b = u8::decode(buf)?;
                    size = (size << 7) | (b & 0x7F) as u32;
                    if b & 0x80 == 0 {
                        break;
                    }
                    if index == 3 {
                        return Err(Error::InvalidSize);
                    }
                }

                match tag {
                    $(
                        $name::TAG => Ok($name::decode_exact(buf, size as _)?.into()),
                    )*
                    _ => Ok(Descriptor::Unknown(tag, Vec::decode_exact(buf, size as _)?)),
                }
            }
        }

        impl DecodeMaybe for Descriptor {
            fn decode_maybe<B: Buf>(buf: &mut B) -> Result<Option<Self>> {
                match buf.has_remaining() {
                    true => Descriptor::decode(buf).map(Some),
                    false => Ok(None),
                }
            }
        }

        impl Encode for Descriptor {
            fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
                // TODO This is inefficient; we could compute the size upfront.
                let mut tmp = Vec::new();

                match self {
                    $(
                        Descriptor::$name(t) => {
                            $name::TAG.encode(buf)?;
                            t.encode(&mut tmp)?;
                        },
                    )*
                    Descriptor::Unknown(tag, data) => {
                        tag.encode(buf)?;
                        data.encode(&mut tmp)?;
                    },
                };

                let mut size = u32::try_from(tmp.len()).map_err(|_| Error::InvalidSize)?;
                if size > 0x0FFF_FFFF {
                    return Err(Error::InvalidSize);
                }

                let mut encoded = [0u8; 4];
                let mut index = encoded.len();
                loop {
                    index -= 1;
                    encoded[index] = (size & 0x7F) as u8;
                    size >>= 7;
                    if size == 0 {
                        break;
                    }
                }
                for index in index..encoded.len() {
                    let continuation = index + 1 < encoded.len();
                    (encoded[index] | u8::from(continuation) << 7).encode(buf)?;
                }

                tmp.encode(buf)
            }
        }

        impl Descriptor {
            pub const fn tag(&self) -> u8 {
                match self {
                    $(
                        Descriptor::$name(_) => $name::TAG,
                    )*
                    Descriptor::Unknown(tag, _) => *tag,
                }
            }
        }

        $(
            impl From<$name> for Descriptor {
                fn from(desc: $name) -> Self {
                    Descriptor::$name(desc)
                }
            }
        )*
    };
}

descriptors! {
    EsDescriptor,
    DecoderConfig,
    DecoderSpecific,
    SLConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EsDescriptor {
    pub es_id: u16,

    pub depends_on_es_id: Option<u16>,
    pub url: Option<Vec<u8>>,
    pub ocr_es_id: Option<u16>,
    pub stream_priority: u8,

    pub dec_config: DecoderConfig,
    pub sl_config: SLConfig,
}

impl EsDescriptor {
    pub const TAG: u8 = 0x03;
}

impl Decode for EsDescriptor {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        let es_id = u16::decode(buf)?;
        let flags = u8::decode(buf)?;
        let stream_priority = flags & 0x1F;
        let depends_on_es_id = (flags & 0x80 != 0).then(|| u16::decode(buf)).transpose()?;
        let url = if flags & 0x40 != 0 {
            let size = u8::decode(buf)? as usize;
            Some(Vec::decode_exact(buf, size)?)
        } else {
            None
        };
        let ocr_es_id = (flags & 0x20 != 0).then(|| u16::decode(buf)).transpose()?;

        let mut dec_config = None;
        let mut sl_config = None;

        while let Some(desc) = Descriptor::decode_maybe(buf)? {
            match desc {
                Descriptor::DecoderConfig(desc) => dec_config = Some(desc),
                Descriptor::SLConfig(desc) => sl_config = Some(desc),
                Descriptor::Unknown(tag, _) => tracing::warn!("unknown descriptor: {:02X}", tag),
                desc => return Err(Error::UnexpectedDescriptor(desc.tag())),
            }
        }

        Ok(EsDescriptor {
            es_id,
            depends_on_es_id,
            url,
            ocr_es_id,
            stream_priority,
            dec_config: dec_config.ok_or(Error::MissingDescriptor(DecoderConfig::TAG))?,
            sl_config: sl_config.ok_or(Error::MissingDescriptor(SLConfig::TAG))?,
        })
    }
}

impl Encode for EsDescriptor {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        if self.stream_priority > 0x1F {
            return Err(Error::InvalidSize);
        }

        let url_size = self
            .url
            .as_ref()
            .map(Vec::len)
            .map(u8::try_from)
            .transpose()
            .map_err(|_| Error::InvalidSize)?;

        self.es_id.encode(buf)?;
        let flags = u8::from(self.depends_on_es_id.is_some()) << 7
            | u8::from(self.url.is_some()) << 6
            | u8::from(self.ocr_es_id.is_some()) << 5
            | self.stream_priority;
        flags.encode(buf)?;
        self.depends_on_es_id.encode(buf)?;
        if let (Some(url), Some(size)) = (&self.url, url_size) {
            size.encode(buf)?;
            url.encode(buf)?;
        }
        self.ocr_es_id.encode(buf)?;

        Descriptor::from(self.dec_config).encode(buf)?;
        Descriptor::from(self.sl_config).encode(buf)?;

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DecoderConfig {
    pub object_type_indication: u8,
    pub stream_type: u8,
    pub up_stream: u8,
    pub buffer_size_db: u24,
    pub max_bitrate: u32,
    pub avg_bitrate: u32,
    pub dec_specific: DecoderSpecific,
}

impl DecoderConfig {
    pub const TAG: u8 = 0x04;
}

impl Decode for DecoderConfig {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        let object_type_indication = u8::decode(buf)?;
        let byte_a = u8::decode(buf)?;
        let stream_type = (byte_a & 0xFC) >> 2;
        let up_stream = byte_a & 0x02;
        let buffer_size_db = u24::decode(buf)?;
        let max_bitrate = u32::decode(buf)?;
        let avg_bitrate = u32::decode(buf)?;

        let mut dec_specific = None;

        while let Some(desc) = Descriptor::decode_maybe(buf)? {
            match desc {
                Descriptor::DecoderSpecific(desc) => dec_specific = Some(desc),
                Descriptor::Unknown(tag, _) => tracing::warn!("unknown descriptor: {:02X}", tag),
                desc => return Err(Error::UnexpectedDescriptor(desc.tag())),
            }
        }

        Ok(DecoderConfig {
            object_type_indication,
            stream_type,
            up_stream,
            buffer_size_db,
            max_bitrate,
            avg_bitrate,
            dec_specific: dec_specific.ok_or(Error::MissingDescriptor(DecoderSpecific::TAG))?,
        })
    }
}

impl Encode for DecoderConfig {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.object_type_indication.encode(buf)?;
        ((self.stream_type << 2) + (self.up_stream & 0x02) + 1).encode(buf)?; // 1 reserved
        self.buffer_size_db.encode(buf)?;
        self.max_bitrate.encode(buf)?;
        self.avg_bitrate.encode(buf)?;

        Descriptor::from(self.dec_specific).encode(buf)?;

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DecoderSpecific {
    pub profile: u8,
    pub freq_index: u8,
    pub sample_rate: Option<u32>,
    pub chan_conf: u8,
}

impl DecoderSpecific {
    pub const TAG: u8 = 0x05;
}

impl Decode for DecoderSpecific {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        let size = buf.remaining();
        let data = buf.slice(size);
        let mut bit_offset = 0;

        let mut profile = decode_bits(data, &mut bit_offset, 5)? as u8;
        if profile == 31 {
            profile = 32 + decode_bits(data, &mut bit_offset, 6)? as u8;
        }

        let freq_index = decode_bits(data, &mut bit_offset, 4)? as u8;
        let sample_rate = if freq_index == 15 {
            Some(decode_bits(data, &mut bit_offset, 24)?)
        } else {
            None
        };
        let chan_conf = decode_bits(data, &mut bit_offset, 4)? as u8;
        buf.advance(size);

        Ok(DecoderSpecific {
            profile,
            freq_index,
            sample_rate,
            chan_conf,
        })
    }
}

impl Encode for DecoderSpecific {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        if self.profile == 31
            || self.profile > 95
            || self.freq_index > 15
            || self.chan_conf > 15
            || (self.freq_index == 15) != self.sample_rate.is_some()
            || self.sample_rate.is_some_and(|rate| rate > 0xFF_FFFF)
        {
            return Err(Error::InvalidSize);
        }

        let mut encoded = Vec::new();
        let mut bit_offset = 0;
        if self.profile < 31 {
            encode_bits(&mut encoded, &mut bit_offset, self.profile as u32, 5);
        } else {
            encode_bits(&mut encoded, &mut bit_offset, 31, 5);
            encode_bits(&mut encoded, &mut bit_offset, (self.profile - 32) as u32, 6);
        }
        encode_bits(&mut encoded, &mut bit_offset, self.freq_index as u32, 4);
        if let Some(sample_rate) = self.sample_rate {
            encode_bits(&mut encoded, &mut bit_offset, sample_rate, 24);
        }
        encode_bits(&mut encoded, &mut bit_offset, self.chan_conf as u32, 4);
        encoded.encode(buf)?;

        Ok(())
    }
}

fn decode_bits(data: &[u8], bit_offset: &mut usize, count: usize) -> Result<u32> {
    if data.len() * 8 < *bit_offset + count {
        return Err(Error::OutOfBounds);
    }

    let mut value = 0;
    for _ in 0..count {
        let byte = data[*bit_offset / 8];
        let bit = (byte >> (7 - *bit_offset % 8)) & 1;
        value = (value << 1) | u32::from(bit);
        *bit_offset += 1;
    }
    Ok(value)
}

fn encode_bits(encoded: &mut Vec<u8>, bit_offset: &mut usize, value: u32, count: usize) {
    for shift in (0..count).rev() {
        if (*bit_offset).is_multiple_of(8) {
            encoded.push(0);
        }
        encoded[*bit_offset / 8] |= ((value >> shift) as u8 & 1) << (7 - *bit_offset % 8);
        *bit_offset += 1;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SLConfig {}

impl SLConfig {
    pub const TAG: u8 = 0x06;
}

impl Decode for SLConfig {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        u8::decode(buf)?; // pre-defined
        Ok(SLConfig {})
    }
}

impl Encode for SLConfig {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        2u8.encode(buf)?; // pre-defined
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_size_uses_most_significant_group_first() {
        let descriptor = Descriptor::Unknown(0x7F, vec![0xAA; 200]);
        let mut encoded = Vec::new();
        descriptor.encode(&mut encoded).unwrap();

        assert_eq!(&encoded[..3], &[0x7F, 0x81, 0x48]);
        let mut encoded = encoded.as_slice();
        assert_eq!(Descriptor::decode(&mut encoded).unwrap(), descriptor);
        assert!(encoded.is_empty());
    }

    #[test]
    fn empty_descriptor_has_a_size_byte() {
        let descriptor = Descriptor::Unknown(0x7F, Vec::new());
        let mut encoded = Vec::new();
        descriptor.encode(&mut encoded).unwrap();

        assert_eq!(encoded, [0x7F, 0x00]);
        assert_eq!(
            Descriptor::decode(&mut encoded.as_slice()).unwrap(),
            descriptor
        );
    }

    #[test]
    fn descriptor_size_rejects_more_than_four_bytes() {
        let mut encoded = &[0x7F, 0x80, 0x80, 0x80, 0x80][..];
        assert!(matches!(
            Descriptor::decode(&mut encoded),
            Err(Error::InvalidSize)
        ));
    }

    #[test]
    fn es_descriptor_flags_round_trip() {
        let descriptor = Descriptor::from(EsDescriptor {
            es_id: 0x1234,
            depends_on_es_id: Some(0x5678),
            url: Some(b"https://example.com/audio".to_vec()),
            ocr_es_id: Some(0x9ABC),
            stream_priority: 17,
            dec_config: DecoderConfig::default(),
            sl_config: SLConfig::default(),
        });

        descriptor.assert_encode_decode();
    }

    #[test]
    fn extended_audio_object_type_and_channel_config_round_trip() {
        let config = DecoderSpecific {
            profile: 42,
            freq_index: 4,
            sample_rate: None,
            chan_conf: 9,
        };
        let mut encoded = Vec::new();
        config.encode(&mut encoded).unwrap();

        assert_eq!(encoded, [0xF9, 0x49, 0x20]);
        assert_eq!(
            DecoderSpecific::decode(&mut encoded.as_slice()).unwrap(),
            config
        );
    }

    #[test]
    fn explicit_sample_rate_round_trip() {
        let config = DecoderSpecific {
            profile: 2,
            freq_index: 15,
            sample_rate: Some(48_000),
            chan_conf: 2,
        };
        let mut encoded = Vec::new();
        config.encode(&mut encoded).unwrap();

        assert_eq!(encoded, [0x17, 0x80, 0x5D, 0xC0, 0x10]);
        assert_eq!(
            DecoderSpecific::decode(&mut encoded.as_slice()).unwrap(),
            config
        );
    }
}
