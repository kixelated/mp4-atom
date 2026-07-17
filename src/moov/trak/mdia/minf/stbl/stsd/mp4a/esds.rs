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

                // ISO/IEC 14496-1 expandable size: base-128, most-significant
                // 7-bit group first, terminated by a byte with the continuation
                // bit clear. It spans at most 4 bytes.
                let mut size: u32 = 0;
                let mut complete = false;
                for _ in 0..4 {
                    let b = u8::decode(buf)?;
                    size = (size << 7) | (b & 0x7F) as u32;
                    if b & 0x80 == 0 {
                        complete = true;
                        break;
                    }
                }
                if !complete {
                    // The continuation bit was still set after 4 bytes; rather than
                    // silently truncating (and desyncing the parse) treat it as invalid.
                    return Err(Error::InvalidSize);
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

                let size = tmp.len() as u32;
                if size >> 28 != 0 {
                    // The size field spans at most 4 base-128 groups (28 bits).
                    return Err(Error::InvalidSize);
                }

                // ISO/IEC 14496-1 expandable size: base-128, most-significant 7-bit
                // group first, with the continuation bit set on all but the last byte.
                // (A size of 0 still emits a single 0x00 byte.)
                let groups = [
                    (size >> 21) as u8 & 0x7F,
                    (size >> 14) as u8 & 0x7F,
                    (size >> 7) as u8 & 0x7F,
                    size as u8 & 0x7F,
                ];
                let start = groups[..3].iter().position(|&g| g != 0).unwrap_or(3);
                for (i, &g) in groups.iter().enumerate().skip(start) {
                    let cont = if i < 3 { 0x80 } else { 0 };
                    (g | cont).encode(buf)?;
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

    /// 5-bit stream priority from the ES_Descriptor flags byte.
    pub stream_priority: u8,

    /// Present when the streamDependenceFlag is set.
    pub depends_on_es_id: Option<u16>,

    /// Present when the URL_Flag is set.
    pub url: Option<String>,

    /// Present when the OCRstreamFlag is set.
    pub ocr_es_id: Option<u16>,

    pub dec_config: DecoderConfig,
    pub sl_config: SLConfig,
}

impl EsDescriptor {
    pub const TAG: u8 = 0x03;
}

impl Decode for EsDescriptor {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        let es_id = u16::decode(buf)?;

        // streamDependenceFlag (1) | URL_Flag (1) | OCRstreamFlag (1) | streamPriority (5)
        let flags = u8::decode(buf)?;
        let stream_priority = flags & 0x1F;

        let depends_on_es_id = if flags & 0x80 != 0 {
            Some(u16::decode(buf)?)
        } else {
            None
        };

        let url = if flags & 0x40 != 0 {
            let len = u8::decode(buf)? as usize;
            let bytes = Vec::<u8>::decode_exact(buf, len)?;
            Some(String::from_utf8(bytes).map_err(|err| Error::InvalidString(err.to_string()))?)
        } else {
            None
        };

        let ocr_es_id = if flags & 0x20 != 0 {
            Some(u16::decode(buf)?)
        } else {
            None
        };

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
            stream_priority,
            depends_on_es_id,
            url,
            ocr_es_id,
            dec_config: dec_config.ok_or(Error::MissingDescriptor(DecoderConfig::TAG))?,
            sl_config: sl_config.ok_or(Error::MissingDescriptor(SLConfig::TAG))?,
        })
    }
}

impl Encode for EsDescriptor {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.es_id.encode(buf)?;

        let mut flags = self.stream_priority & 0x1F;
        if self.depends_on_es_id.is_some() {
            flags |= 0x80;
        }
        if self.url.is_some() {
            flags |= 0x40;
        }
        if self.ocr_es_id.is_some() {
            flags |= 0x20;
        }
        flags.encode(buf)?;

        if let Some(depends_on_es_id) = self.depends_on_es_id {
            depends_on_es_id.encode(buf)?;
        }
        if let Some(url) = &self.url {
            let bytes = url.as_bytes();
            let len = u8::try_from(bytes.len()).map_err(|_| Error::OutOfMemory)?;
            len.encode(buf)?;
            buf.append_slice(bytes);
        }
        if let Some(ocr_es_id) = self.ocr_es_id {
            ocr_es_id.encode(buf)?;
        }

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
    /// audioObjectType (AAC profile). Values >= 32 use the 6-bit escape code.
    pub profile: u8,

    /// samplingFrequencyIndex; 15 (0x0F) signals an explicit `sample_rate`.
    pub freq_index: u8,

    /// Explicit 24-bit sample rate, only present when `freq_index == 15`.
    pub sample_rate: Option<u32>,

    /// channelConfiguration.
    pub chan_conf: u8,
}

impl DecoderSpecific {
    pub const TAG: u8 = 0x05;
}

impl Decode for DecoderSpecific {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        // AudioSpecificConfig is a bit-packed structure (ISO/IEC 14496-3 1.6.2.1),
        // so read the whole payload and walk it bit by bit.
        let mut bytes = Vec::new();
        while buf.has_remaining() {
            bytes.push(u8::decode(buf)?);
        }
        let mut bits = BitReader::new(&bytes);

        // audioObjectType: 5 bits, escaping to 32 + 6 bits when it reads 31.
        let mut profile = bits.read(5)? as u8;
        if profile == 31 {
            profile = 32 + bits.read(6)? as u8;
        }

        let freq_index = bits.read(4)? as u8;
        let sample_rate = if freq_index == 15 {
            Some(bits.read(24)?)
        } else {
            None
        };

        let chan_conf = bits.read(4)? as u8;

        // Any trailing bits (GASpecificConfig etc.) are not represented.

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
        let mut bits = BitWriter::default();

        // audioObjectType: 5 bits, or the 31 escape followed by 32 + 6 bits.
        if self.profile < 31 {
            bits.write(self.profile as u32, 5);
        } else {
            // 31 is the escape marker itself, so it can't be encoded directly and
            // the 6-bit extension caps the object type at 32 + 63.
            if self.profile == 31 || self.profile > 32 + 63 {
                return Err(Error::Unsupported("invalid audio object type"));
            }
            bits.write(31, 5);
            bits.write((self.profile - 32) as u32, 6);
        }

        bits.write(self.freq_index as u32, 4);
        if self.freq_index == 15 {
            bits.write(self.sample_rate.unwrap_or(0), 24);
        }
        bits.write(self.chan_conf as u32, 4);

        buf.append_slice(&bits.finish());

        Ok(())
    }
}

/// Reads big-endian bit fields (MSB first) out of a byte slice.
struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Read up to 32 bits, MSB first.
    fn read(&mut self, count: usize) -> Result<u32> {
        let mut value = 0u32;
        for _ in 0..count {
            let byte = self.pos / 8;
            if byte >= self.data.len() {
                return Err(Error::OutOfBounds);
            }
            let bit = (self.data[byte] >> (7 - (self.pos % 8))) & 1;
            value = (value << 1) | bit as u32;
            self.pos += 1;
        }
        Ok(value)
    }
}

/// Writes big-endian bit fields (MSB first), zero-padding the final byte.
#[derive(Default)]
struct BitWriter {
    data: Vec<u8>,
    pos: usize,
}

impl BitWriter {
    /// Write the low `count` bits of `value`, MSB first.
    fn write(&mut self, value: u32, count: usize) {
        for i in (0..count).rev() {
            if self.pos.is_multiple_of(8) {
                self.data.push(0);
            }
            let bit = ((value >> i) & 1) as u8;
            let byte = self.pos / 8;
            self.data[byte] |= bit << (7 - (self.pos % 8));
            self.pos += 1;
        }
    }

    fn finish(self) -> Vec<u8> {
        self.data
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

    // A helper that encodes then decodes and asserts the value round-trips.
    fn round_trip<T: Encode + Decode + std::fmt::Debug + PartialEq>(value: &T) {
        value.assert_encode_decode();
    }

    #[test]
    fn descriptor_size_varint() {
        // Sizes that need 1, 2 and 3 length bytes, plus the empty-payload edge case.
        for len in [0usize, 1, 127, 128, 200, 16383, 16384] {
            let desc = Descriptor::Unknown(0x99, vec![0xAB; len]);

            let mut buf = Vec::new();
            desc.encode(&mut buf).unwrap();

            let mut cursor = std::io::Cursor::new(&buf);
            let decoded = Descriptor::decode(&mut cursor).unwrap();
            assert_eq!(desc, decoded, "size {len} did not round-trip");
        }
    }

    #[test]
    fn descriptor_size_200_bytes() {
        // Regression for the reversed varint: 200 must encode as 0x81 0x48, not 0xC8 0x01.
        let desc = Descriptor::Unknown(0x99, vec![0; 200]);
        let mut buf = Vec::new();
        desc.encode(&mut buf).unwrap();
        assert_eq!(&buf[..3], &[0x99, 0x81, 0x48]);
    }

    #[test]
    fn descriptor_size_desync_errors() {
        // A size field whose 4th byte still has the continuation bit set is invalid.
        let bytes = [0x99u8, 0x80, 0x80, 0x80, 0x80, 0x00];
        let mut cursor = std::io::Cursor::new(&bytes);
        assert!(matches!(
            Descriptor::decode(&mut cursor),
            Err(Error::InvalidSize)
        ));
    }

    #[test]
    fn audio_specific_config_basic() {
        round_trip(&DecoderSpecific {
            profile: 2, // AAC-LC
            freq_index: 4,
            sample_rate: None,
            chan_conf: 2,
        });
    }

    #[test]
    fn audio_specific_config_extended_profile() {
        // audioObjectType 42 (xHE-AAC / USAC) uses the 5-bit escape + 6-bit extension.
        round_trip(&DecoderSpecific {
            profile: 42,
            freq_index: 4,
            sample_rate: None,
            chan_conf: 6,
        });
    }

    #[test]
    fn audio_specific_config_explicit_sample_rate() {
        // freq_index 15 escapes to an explicit 24-bit sample rate.
        round_trip(&DecoderSpecific {
            profile: 2,
            freq_index: 15,
            sample_rate: Some(96_000),
            chan_conf: 2,
        });
    }

    #[test]
    fn audio_specific_config_extended_profile_explicit_rate() {
        round_trip(&DecoderSpecific {
            profile: 42,
            freq_index: 15,
            sample_rate: Some(352_800),
            chan_conf: 1,
        });
    }

    #[test]
    fn es_descriptor_flags() {
        // streamDependence / URL / OCR flags must round-trip their trailing fields.
        round_trip(&EsDescriptor {
            es_id: 2,
            stream_priority: 5,
            depends_on_es_id: Some(7),
            url: Some("http://example.com".into()),
            ocr_es_id: Some(9),
            dec_config: DecoderConfig {
                object_type_indication: 0x40,
                stream_type: 5,
                dec_specific: DecoderSpecific {
                    profile: 2,
                    freq_index: 4,
                    chan_conf: 2,
                    ..Default::default()
                },
                ..Default::default()
            },
            sl_config: SLConfig {},
        });
    }
}
