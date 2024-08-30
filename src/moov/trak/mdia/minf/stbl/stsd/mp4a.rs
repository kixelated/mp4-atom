use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mp4a {
    pub data_reference_index: u16,
    pub channelcount: u16,
    pub samplesize: u16,
    pub samplerate: FixedPoint<u16>,
    pub esds: Option<Esds>,
}

impl Default for Mp4a {
    fn default() -> Self {
        Self {
            data_reference_index: 0,
            channelcount: 2,
            samplesize: 16,
            samplerate: 48000.into(),
            esds: Some(Esds::default()),
        }
    }
}

impl Atom for Mp4a {
    const KIND: FourCC = FourCC::new(b"mp4a");

    fn decode_atom<B: Buf>(buf: &mut B) -> Result<Self> {
        u32::decode(buf)?; // reserved
        u16::decode(buf)?; // reserved
        let data_reference_index = buf.decode()?;
        let version = u16::decode(buf)?;
        u16::decode(buf)?; // reserved
        u32::decode(buf)?; // reserved
        let channelcount = buf.decode()?;
        let samplesize = buf.decode()?;
        u32::decode(buf)?; // pre-defined, reserved
        let samplerate = buf.decode()?;

        if version == 1 {
            // Skip QTFF
            u64::decode(buf)?;
            u64::decode(buf)?;
        }

        let mut esds = None;

        // Find esds in mp4a or wave
        while let Some(atom) = buf.decode()? {
            match atom {
                Any::Esds(atom) => esds = atom.into(),
                _ => tracing::warn!("unknown atom: {:?}", atom),
            }
        }

        Ok(Mp4a {
            data_reference_index,
            channelcount,
            samplesize,
            samplerate,
            esds,
        })
    }

    fn encode_atom(&self, buf: &mut BytesMut) -> Result<()> {
        0u32.encode(buf)?; // reserved
        0u16.encode(buf)?; // reserved
        self.data_reference_index.encode(buf)?;
        0u16.encode(buf)?; // version
        0u16.encode(buf)?; // reserved
        0u32.encode(buf)?; // reserved
        self.channelcount.encode(buf)?;
        self.samplesize.encode(buf)?;
        0u32.encode(buf)?; // reserved
        self.samplerate.encode(buf)?;

        self.esds.encode(buf)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Esds {
    pub es_desc: ESDescriptor,
}

impl AtomExt for Esds {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"esds");

    fn decode_atom_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let mut es_desc = None;

        while let Some(desc) = Option::<DescriptorHeader>::decode(buf)? {
            let mut buf = buf.take(desc.size);
            match desc.tag {
                0x03 => es_desc = ESDescriptor::decode(&mut buf)?.into(),
                _ => todo!("Esds tag: {:02X}", desc.tag),
            }
        }

        Ok(Esds {
            es_desc: es_desc.ok_or(Error::MissingDescriptor)?,
        })
    }

    fn encode_atom_ext(&self, buf: &mut BytesMut) -> Result<()> {
        // TODO also include the descriptor header
        self.es_desc.encode(buf)?;

        Ok(())
    }
}

pub struct DescriptorHeader {
    pub tag: u8,
    pub size: usize,
}

impl Decode for DescriptorHeader {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        let tag = u8::decode(buf)?;

        let mut size: u32 = 0;
        for _ in 0..4 {
            let b = u8::decode(buf)?;
            size = (size << 7) | (b & 0x7F) as u32;
            if b & 0x80 == 0 {
                break;
            }
        }

        Ok(Self {
            tag,
            size: size as usize,
        })
    }
}

impl Encode for DescriptorHeader {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        self.tag.encode(buf)?;

        let mut size = self.size as u32;
        while size > 0 {
            let mut b = (size & 0x7F) as u8;
            size >>= 7;
            if size > 0 {
                b |= 0x80;
            }
            (b).encode(buf)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ESDescriptor {
    pub es_id: u16,

    pub dec_config: DecoderConfigDescriptor,
    pub sl_config: SLConfigDescriptor,
}

impl Decode for ESDescriptor {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        let es_id = buf.decode()?;
        u8::decode(buf)?; // XXX flags must be 0

        let mut dec_config = None;
        let mut sl_config = None;

        while let Some(desc) = Option::<DescriptorHeader>::decode(buf)? {
            let buf = &mut buf.take(desc.size);
            match desc.tag {
                0x04 => dec_config = DecoderConfigDescriptor::decode(buf)?.into(),
                0x06 => sl_config = SLConfigDescriptor::decode(buf)?.into(),
                _ => todo!("ESDescriptor tag: {:02X}", desc.tag),
            }
        }

        Ok(ESDescriptor {
            es_id,
            dec_config: dec_config.unwrap_or_default(),
            sl_config: sl_config.unwrap_or_default(),
        })
    }
}

impl Encode for ESDescriptor {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        // TODO write the header

        self.es_id.encode(buf)?;
        0u8.encode(buf)?;

        self.dec_config.encode(buf)?;
        self.sl_config.encode(buf)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DecoderConfigDescriptor {
    pub object_type_indication: u8,
    pub stream_type: u8,
    pub up_stream: u8,
    pub buffer_size_db: u24,
    pub max_bitrate: u32,
    pub avg_bitrate: u32,

    pub dec_specific: DecoderSpecificDescriptor,
}

impl Decode for DecoderConfigDescriptor {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        let object_type_indication = u8::decode(buf)?;
        let byte_a = u8::decode(buf)?;
        let stream_type = (byte_a & 0xFC) >> 2;
        let up_stream = byte_a & 0x02;
        let buffer_size_db = u24::decode(buf)?;
        let max_bitrate = u32::decode(buf)?;
        let avg_bitrate = u32::decode(buf)?;

        let mut dec_specific = None;

        while let Some(desc) = Option::<DescriptorHeader>::decode(buf)? {
            let buf = &mut buf.take(desc.size);
            match desc.tag {
                0x05 => dec_specific = DecoderSpecificDescriptor::decode(buf)?.into(),
                _ => todo!("DecoderConfigDescriptor tag: {:02X}", desc.tag),
            }
        }

        Ok(DecoderConfigDescriptor {
            object_type_indication,
            stream_type,
            up_stream,
            buffer_size_db,
            max_bitrate,
            avg_bitrate,
            dec_specific: dec_specific.unwrap_or_default(),
        })
    }
}

impl Encode for DecoderConfigDescriptor {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        // TODO write the header

        self.object_type_indication.encode(buf)?;
        ((self.stream_type << 2) + (self.up_stream & 0x02) + 1).encode(buf)?; // 1 reserved
        self.buffer_size_db.encode(buf)?;
        self.max_bitrate.encode(buf)?;
        self.avg_bitrate.encode(buf)?;

        self.dec_specific.encode(buf)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DecoderSpecificDescriptor {
    pub profile: u8,
    pub freq_index: u8,
    pub chan_conf: u8,
}

fn get_audio_object_type(byte_a: u8, byte_b: u8) -> u8 {
    let mut profile = byte_a >> 3;
    if profile == 31 {
        profile = 32 + ((byte_a & 7) | (byte_b >> 5));
    }

    profile
}

fn decode_chan_conf<B: Buf>(
    buf: &mut B,
    byte_b: u8,
    freq_index: u8,
    extended_profile: bool,
) -> Result<u8> {
    let chan_conf;
    if freq_index == 15 {
        // Skip the 24 bit sample rate
        let sample_rate = u24::decode(buf)?;
        chan_conf = ((u32::from(sample_rate) >> 4) & 0x0F) as u8;
    } else if extended_profile {
        let byte_c = u8::decode(buf)?;
        chan_conf = (byte_b & 1) | (byte_c & 0xE0);
    } else {
        chan_conf = (byte_b >> 3) & 0x0F;
    }

    Ok(chan_conf)
}

impl Decode for DecoderSpecificDescriptor {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        let byte_a = u8::decode(buf)?;
        let byte_b = u8::decode(buf)?;
        let profile = get_audio_object_type(byte_a, byte_b);
        let freq_index;
        let chan_conf;
        if profile > 31 {
            freq_index = (byte_b >> 1) & 0x0F;
            chan_conf = decode_chan_conf(buf, byte_b, freq_index, true)?;
        } else {
            freq_index = ((byte_a & 0x07) << 1) + (byte_b >> 7);
            chan_conf = decode_chan_conf(buf, byte_b, freq_index, false)?;
        }

        Ok(DecoderSpecificDescriptor {
            profile,
            freq_index,
            chan_conf,
        })
    }
}

impl Encode for DecoderSpecificDescriptor {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        // TODO write the header

        ((self.profile << 3) + (self.freq_index >> 1)).encode(buf)?;
        ((self.freq_index << 7) + (self.chan_conf << 3)).encode(buf)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SLConfigDescriptor {}

impl Decode for SLConfigDescriptor {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        u8::decode(buf)?; // pre-defined

        Ok(SLConfigDescriptor {})
    }
}

impl Encode for SLConfigDescriptor {
    fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        // TODO write the header

        2u8.encode(buf)?; // pre-defined

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_mp4a() {
        let expected = Mp4a {
            data_reference_index: 1,
            channelcount: 2,
            samplesize: 16,
            samplerate: 48000.into(),
            esds: Some(Esds {
                es_desc: ESDescriptor {
                    es_id: 2,
                    dec_config: DecoderConfigDescriptor {
                        object_type_indication: 0x40,
                        stream_type: 0x05,
                        up_stream: 0,
                        buffer_size_db: Default::default(),
                        max_bitrate: 67695,
                        avg_bitrate: 67695,
                        dec_specific: DecoderSpecificDescriptor {
                            profile: 2,
                            freq_index: 3,
                            chan_conf: 1,
                        },
                    },
                    sl_config: SLConfigDescriptor::default(),
                },
            }),
        };
        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let decoded = Mp4a::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_mp4a_no_esds() {
        let expected = Mp4a {
            data_reference_index: 1,
            channelcount: 2,
            samplesize: 16,
            samplerate: 48000.into(),
            esds: None,
        };
        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let decoded = Mp4a::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
