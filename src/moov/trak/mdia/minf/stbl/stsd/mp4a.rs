use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mp4a {
    pub data_reference_index: u16,
    pub channelcount: u16,
    pub samplesize: u16,
    pub samplerate: Ratio<u16>,
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

impl Mp4a {
    pub fn new(config: &AacConfig) -> Self {
        Self {
            data_reference_index: 1,
            channelcount: config.chan_conf as u16,
            samplesize: 16,
            samplerate: config.freq_index.freq().into(),
            esds: Some(Esds::new(config)),
        }
    }
}

impl Atom for Mp4a {
    const KIND: FourCC = FourCC::new(b"mp4a");

    fn decode_atom(buf: &mut Buf) -> Result<Self> {
        u32::decode(buf)?; // reserved
        u16::decode(buf)?; // reserved
        let data_reference_index = buf.decode()?;
        let version = buf.u16()?;
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

        // Find esds in mp4a or wave
        let mut esds = None;

        while let Some(atom) = buf.decode()? {
            match atom {
                Any::Esds(atom) => {
                    esds.replace(atom);
                    break;
                }
                Any::Wave(atom) => {
                    // Typically contains frma, mp4a, esds, and a terminator atom
                }
                _ => return Err(Error::UnexpectedBox(atom.kind())),
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

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        buf.u32(0)?; // reserved
        buf.u16(0)?; // reserved
        self.data_reference_index.encode(buf)?;
        buf.u16(0)?; // version
        buf.u16(0)?; // reserved
        buf.u32(0)?; // reserved
        self.channelcount.encode(buf)?;
        self.samplesize.encode(buf)?;
        buf.u32(0)?; // reserved
        self.samplerate.encode(buf)?;

        self.esds.encode(buf)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Esds {
    pub es_desc: ESDescriptor,
}

impl Esds {
    pub fn new(config: &AacConfig) -> Self {
        Self {
            es_desc: ESDescriptor::new(config),
        }
    }
}

impl AtomExt for Esds {
  type Ext = ();

  const KIND: FourCC = FourCC::new(b"esds");

fn decode_atom(buf: &mut Buf, _ext: ()) -> Result<Self> {
        let mut es_desc = None;

        while let Some(atom) = buf.decode()? {
            let (desc_tag, desc_size) = read_desc(reader)?;
            match desc_tag {
                0x03 => {
                    es_desc = Some(ESDescriptor::read_desc(reader, desc_size)?);
                }
                _ => break,
            }
        }

        Ok(Esds {
            es_desc: es_desc.ok_or(Error::MissingDescriptor)?,
        })
    }

fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        self.es_desc.write_desc(writer)?;

        Ok(())
    }
}

pub struct Descriptor {
    pub tag: u8,
    pub size: u32,
}

impl Decode for Descriptor {
    fn decode(buf: &mut Buf) -> Result<Self> {
        let tag = buf.u8()?;

        let mut size: u32 = 0;
        for _ in 0..4 {
            let b = buf.u8()?;
            size = (size << 7) | (b & 0x7F) as u32;
            if b & 0x80 == 0 {
                break;
            }
        }

        Ok(Descriptor { tag, size })
    }
}

impl Encode for Descriptor {
    fn encode(&self, buf: &mut BufMut) -> Result<()> {
        buf.u8(self.tag)?;

        let mut size = self.size;
        let mut nbytes = 0;
        while size > 0 {
            let mut b = (size & 0x7F) as u8;
            size >>= 7;
            if size > 0 {
                b |= 0x80;
            }
            buf.u8(b)?;
            nbytes += 1;
        }

        Ok(nbytes)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ESDescriptor {
    pub es_id: u16,

    pub dec_config: DecoderConfigDescriptor,
    pub sl_config: SLConfigDescriptor,
}

impl ESDescriptor {
    pub fn new(config: &AacConfig) -> Self {
        Self {
            es_id: 1,
            dec_config: DecoderConfigDescriptor::new(config),
            sl_config: SLConfigDescriptor::new(),
        }
    }
}

impl Descriptor for ESDescriptor {
    fn desc_tag() -> u8 {
        0x03
    }

    fn desc_size() -> u32 {
        3 + 1
            + size_of_length(DecoderConfigDescriptor::desc_size())
            + DecoderConfigDescriptor::desc_size()
            + 1
            + size_of_length(SLConfigDescriptor::desc_size())
            + SLConfigDescriptor::desc_size()
    }
}

impl<R: Read + Seek> ReadDesc<&mut R> for ESDescriptor {
    fn read_desc(reader: &mut R, size: u32) -> Result<Self> {
        let es_id = buf.decode()?;
        buf.u8()?; // XXX flags must be 0

        let mut dec_config = None;
        let mut sl_config = None;

        let end = start + size as u64;
        while let Some(atom) = buf.decode()? {
            let (desc_tag, desc_size) = read_desc(reader)?;
            match desc_tag {
                0x04 => {
                    dec_config = Some(DecoderConfigDescriptor::read_desc(reader, desc_size)?);
                }
                0x06 => {
                    sl_config = Some(SLConfigDescriptor::read_desc(reader, desc_size)?);
                }
                _ => {
                    skip_bytes(reader, desc_size as u64)?;
                }
            }
        }

        Ok(ESDescriptor {
            es_id,
            dec_config: dec_config.unwrap_or_default(),
            sl_config: sl_config.unwrap_or_default(),
        })
    }
}

impl<W: Write> WriteDesc<&mut W> for ESDescriptor {
    fn write_desc(&self, writer: &mut W) -> Result<u32> {
        let size = Self::desc_size();
        write_desc(writer, Self::desc_tag(), size)?;

        self.es_id.encode(buf)?;
        buf.u8(0)?;

        self.dec_config.write_desc(writer)?;
        self.sl_config.write_desc(writer)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DecoderConfigDescriptor {
    pub object_type_indication: u8,
    pub stream_type: u8,
    pub up_stream: u8,
    pub buffer_size_db: u32,
    pub max_bitrate: u32,
    pub avg_bitrate: u32,

    pub dec_specific: DecoderSpecificDescriptor,
}

impl DecoderConfigDescriptor {
    pub fn new(config: &AacConfig) -> Self {
        Self {
            object_type_indication: 0x40, // XXX AAC
            stream_type: 0x05,            // XXX Audio
            up_stream: 0,
            buffer_size_db: 0,
            max_bitrate: config.bitrate, // XXX
            avg_bitrate: config.bitrate,
            dec_specific: DecoderSpecificDescriptor::new(config),
        }
    }
}

impl Descriptor for DecoderConfigDescriptor {
    fn desc_tag() -> u8 {
        0x04
    }

    fn desc_size() -> u32 {
        13 + 1
            + size_of_length(DecoderSpecificDescriptor::desc_size())
            + DecoderSpecificDescriptor::desc_size()
    }
}

impl<R: Read + Seek> ReadDesc<&mut R> for DecoderConfigDescriptor {
    fn read_desc(reader: &mut R, size: u32) -> Result<Self> {
        let object_type_indication = buf.u8()?;
        let byte_a = buf.u8()?;
        let stream_type = (byte_a & 0xFC) >> 2;
        let up_stream = byte_a & 0x02;
        let buffer_size_db = buf.u24()?;
        let max_bitrate = u32::decode(buf)?;
        let avg_bitrate = u32::decode(buf)?;

        let mut dec_specific = None;

        let end = start + size as u64;
        while let Some(atom) = buf.decode()? {
            let (desc_tag, desc_size) = read_desc(reader)?;
            match desc_tag {
                0x05 => {
                    dec_specific = Some(DecoderSpecificDescriptor::read_desc(reader, desc_size)?);
                }
                _ => {
                    skip_bytes(reader, desc_size as u64)?;
                }
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

impl<W: Write> WriteDesc<&mut W> for DecoderConfigDescriptor {
    fn write_desc(&self, writer: &mut W) -> Result<u32> {
        let size = Self::desc_size();
        write_desc(writer, Self::desc_tag(), size)?;

        buf.u8(self.object_type_indication)?;
        buf.u8((self.stream_type << 2) + (self.up_stream & 0x02) + 1)?; // 1 reserved
        self.buffer_size_db.encode(buf)?;
        self.max_bitrate.encode(buf)?;
        self.avg_bitrate.encode(buf)?;

        self.dec_specific.write_desc(writer)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DecoderSpecificDescriptor {
    pub profile: u8,
    pub freq_index: u8,
    pub chan_conf: u8,
}

impl DecoderSpecificDescriptor {
    pub fn new(config: &AacConfig) -> Self {
        Self {
            profile: config.profile as u8,
            freq_index: config.freq_index as u8,
            chan_conf: config.chan_conf as u8,
        }
    }
}

impl Descriptor for DecoderSpecificDescriptor {
    fn desc_tag() -> u8 {
        0x05
    }

    fn desc_size() -> u32 {
        2
    }
}

fn get_audio_object_type(byte_a: u8, byte_b: u8) -> u8 {
    let mut profile = byte_a >> 3;
    if profile == 31 {
        profile = 32 + ((byte_a & 7) | (byte_b >> 5));
    }

    profile
}

fn get_chan_conf<R: Read + Seek>(
    reader: &mut R,
    byte_b: u8,
    freq_index: u8,
    extended_profile: bool,
) -> Result<u8> {
    let chan_conf;
    if freq_index == 15 {
        // Skip the 24 bit sample rate
        let sample_rate = buf.u24()?;
        chan_conf = ((sample_rate >> 4) & 0x0F) as u8;
    } else if extended_profile {
        let byte_c = buf.u8()?;
        chan_conf = (byte_b & 1) | (byte_c & 0xE0);
    } else {
        chan_conf = (byte_b >> 3) & 0x0F;
    }

    Ok(chan_conf)
}

impl<R: Read + Seek> ReadDesc<&mut R> for DecoderSpecificDescriptor {
    fn read_desc(reader: &mut R, _size: u32) -> Result<Self> {
        let byte_a = buf.u8()?;
        let byte_b = buf.u8()?;
        let profile = get_audio_object_type(byte_a, byte_b);
        let freq_index;
        let chan_conf;
        if profile > 31 {
            freq_index = (byte_b >> 1) & 0x0F;
            chan_conf = get_chan_conf(reader, byte_b, freq_index, true)?;
        } else {
            freq_index = ((byte_a & 0x07) << 1) + (byte_b >> 7);
            chan_conf = get_chan_conf(reader, byte_b, freq_index, false)?;
        }

        Ok(DecoderSpecificDescriptor {
            profile,
            freq_index,
            chan_conf,
        })
    }
}

impl<W: Write> WriteDesc<&mut W> for DecoderSpecificDescriptor {
    fn write_desc(&self, writer: &mut W) -> Result<u32> {
        let size = Self::desc_size();
        write_desc(writer, Self::desc_tag(), size)?;

        buf.u8((self.profile << 3) + (self.freq_index >> 1))?;
        buf.u8((self.freq_index << 7) + (self.chan_conf << 3))?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SLConfigDescriptor {}

impl SLConfigDescriptor {
    pub fn new() -> Self {
        SLConfigDescriptor {}
    }
}

impl Descriptor for SLConfigDescriptor {
    fn desc_tag() -> u8 {
        0x06
    }

    fn desc_size() -> u32 {
        1
    }
}

impl<R: Read + Seek> ReadDesc<&mut R> for SLConfigDescriptor {
    fn read_desc(reader: &mut R, _size: u32) -> Result<Self> {
        buf.u8()?; // pre-defined

        Ok(SLConfigDescriptor {})
    }
}

impl<W: Write> WriteDesc<&mut W> for SLConfigDescriptor {
    fn write_desc(&self, writer: &mut W) -> Result<u32> {
        let size = Self::desc_size();
        write_desc(writer, Self::desc_tag(), size)?;

        buf.u8(2)?; // pre-defined
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
                        buffer_size_db: 0,
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
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
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
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Mp4a::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
