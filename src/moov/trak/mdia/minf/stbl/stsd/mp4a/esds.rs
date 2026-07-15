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
        encode_descriptor(EsDescriptor::TAG, &self.es_desc, buf)
    }
}

/// Decode a descriptor body of `size` bytes, tolerating trailing bytes the
/// specific descriptor doesn't consume.
///
/// MPEG-4 (ISO/IEC 14496-1) descriptors are length-prefixed and forward-
/// compatible: the size field is authoritative and a parser skips to the
/// declared end, ignoring extension fields (or writer padding) it doesn't
/// understand. Unlike [`Decode::decode_exact`] this does not `ShortRead` on the
/// leftover — e.g. a 2-byte `SLConfigDescriptor` (`predefined` + a trailing
/// byte) decodes instead of failing the whole `esds`. The `size` slice still
/// bounds the read, so a descriptor can never run past its declared length.
fn decode_descriptor_body<T: Decode, B: Buf>(buf: &mut B, size: usize) -> Result<T> {
    if buf.remaining() < size {
        return Err(Error::OutOfBounds);
    }
    let mut inner = buf.slice(size);
    let res = T::decode(&mut inner)?;
    buf.advance(size);
    Ok(res)
}

/// Encode a typed descriptor — tag, base-128 length prefix, then the body —
/// borrowing `body` rather than taking ownership, so a caller holding a `&T`
/// need not clone it into a [`Descriptor`] just to serialize it.
fn encode_descriptor<T: Encode, B: BufMut>(tag: u8, body: &T, buf: &mut B) -> Result<()> {
    tag.encode(buf)?;

    // TODO This is inefficient; we could compute the size upfront.
    let mut tmp = Vec::new();
    body.encode(&mut tmp)?;

    let mut size = tmp.len() as u32;
    while size > 0 {
        let mut b = (size & 0x7F) as u8;
        size >>= 7;
        if size > 0 {
            b |= 0x80;
        }
        b.encode(buf)?;
    }

    tmp.encode(buf)
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
                for _ in 0..4 {
                    let b = u8::decode(buf)?;
                    size = (size << 7) | (b & 0x7F) as u32;
                    if b & 0x80 == 0 {
                        break;
                    }
                }

                match tag {
                    $(
                        $name::TAG => Ok(decode_descriptor_body::<$name, _>(buf, size as _)?.into()),
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
                match self {
                    $(
                        Descriptor::$name(t) => encode_descriptor($name::TAG, t, buf),
                    )*
                    Descriptor::Unknown(tag, data) => encode_descriptor(*tag, data, buf),
                }
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

    pub dec_config: DecoderConfig,
    pub sl_config: SLConfig,
}

impl EsDescriptor {
    pub const TAG: u8 = 0x03;
}

impl Decode for EsDescriptor {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        let es_id = u16::decode(buf)?;
        u8::decode(buf)?; // XXX flags must be 0

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
            dec_config: dec_config.ok_or(Error::MissingDescriptor(DecoderConfig::TAG))?,
            sl_config: sl_config.ok_or(Error::MissingDescriptor(SLConfig::TAG))?,
        })
    }
}

impl Encode for EsDescriptor {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.es_id.encode(buf)?;
        0u8.encode(buf)?;

        encode_descriptor(DecoderConfig::TAG, &self.dec_config, buf)?;
        encode_descriptor(SLConfig::TAG, &self.sl_config, buf)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DecoderConfig {
    pub object_type_indication: u8,
    pub stream_type: u8,
    pub up_stream: u8,
    pub buffer_size_db: u24,
    pub max_bitrate: u32,
    pub avg_bitrate: u32,
    /// The `DecoderSpecificInfo` (tag 0x05). ISO/IEC 14496-1 makes this
    /// child optional ("if available"): a stream whose config can be derived
    /// in-band (e.g. AAC carried with ADTS headers) legitimately omits it, so a
    /// missing tag 5 must not fail the whole `esds`. `None` means the container
    /// carried no out-of-band config.
    pub dec_specific: Option<DecoderSpecific>,
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
            dec_specific,
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

        if let Some(dec_specific) = &self.dec_specific {
            encode_descriptor(DecoderSpecific::TAG, dec_specific, buf)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DecoderSpecific {
    pub profile: u8,
    pub freq_index: u8,
    pub chan_conf: u8,
    /// The complete `DecoderSpecificInfo` payload. For AAC this is the
    /// AudioSpecificConfig — the `profile` / `freq_index` / `chan_conf` fields
    /// above are a parsed view of its prefix. For a non-AAC object type (e.g.
    /// the MPEG-4 Visual VOS/VOL config carried by an `mp4v` `esds`) the fields
    /// are not meaningful, but the bytes are preserved verbatim so the
    /// descriptor round-trips faithfully. Empty on a hand-constructed AAC
    /// config, in which case [`Encode`] re-derives the 2-byte AudioSpecificConfig
    /// from the fields.
    pub raw: Vec<u8>,
}

impl DecoderSpecific {
    pub const TAG: u8 = 0x05;
}

/// Best-effort parse of the AAC AudioSpecificConfig prefix (`audioObjectType`,
/// `samplingFrequencyIndex`, `channelConfiguration`) from the raw payload.
/// Returns zeros when the payload is too short to parse (e.g. a non-AAC config);
/// the raw bytes remain the source of truth for round-tripping.
fn parse_audio_specific_config(raw: &[u8]) -> (u8, u8, u8) {
    if raw.len() < 2 {
        return (0, 0, 0);
    }
    let byte_a = raw[0];
    let byte_b = raw[1];

    let mut profile = byte_a >> 3;
    if profile == 31 {
        profile = 32 + ((byte_a & 7) | (byte_b >> 5));
    }

    let freq_index = if profile > 31 {
        (byte_b >> 1) & 0x0F
    } else {
        ((byte_a & 0x07) << 1) + (byte_b >> 7)
    };

    let chan_conf = if freq_index == 15 {
        // The 24-bit explicit sample rate precedes the channel config.
        if raw.len() >= 5 {
            let sample_rate =
                (u32::from(raw[2]) << 16) | (u32::from(raw[3]) << 8) | u32::from(raw[4]);
            ((sample_rate >> 4) & 0x0F) as u8
        } else {
            0
        }
    } else if profile > 31 {
        if raw.len() >= 3 {
            (byte_b & 1) | (raw[2] & 0xE0)
        } else {
            0
        }
    } else {
        (byte_b >> 3) & 0x0F
    };

    (profile, freq_index, chan_conf)
}

impl Decode for DecoderSpecific {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        // Capture the complete payload (the descriptor body is already
        // size-bounded by `decode_descriptor_body`) so any object type
        // round-trips; the AAC fields are a best-effort parse of its prefix.
        let raw = Vec::decode(buf)?;
        let (profile, freq_index, chan_conf) = parse_audio_specific_config(&raw);

        Ok(DecoderSpecific {
            profile,
            freq_index,
            chan_conf,
            raw,
        })
    }
}

impl Encode for DecoderSpecific {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        if self.raw.is_empty() {
            // Hand-constructed AAC config with no preserved bytes: re-derive the
            // 2-byte AudioSpecificConfig from the fields.
            ((self.profile << 3) + (self.freq_index >> 1)).encode(buf)?;
            ((self.freq_index << 7) + (self.chan_conf << 3)).encode(buf)?;
        } else {
            // Emit the preserved payload verbatim (faithful for any object type).
            self.raw.encode(buf)?;
        }

        Ok(())
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

    // An `esds` whose `SLConfigDescriptor` (tag 0x06) declares 2 bytes — a
    // `predefined` value plus a trailing byte — instead of the usual 1.
    // `SLConfig::decode` reads only `predefined`, so the strict `decode_exact`
    // used to `ShortRead` and fail the whole `esds` with `UnderDecode(esds)`.
    // ISO 14496-1 makes the descriptor length authoritative, so the trailing
    // byte must be skipped. Descriptor tree: EsDescriptor(0x03) → DecoderConfig
    // (0x04) → DecoderSpecific(0x05, AAC-LC 48 kHz) + SLConfig(0x06, 2 bytes).
    const ESDS_2BYTE_SLCONFIG: &[u8] = &[
        0x00, 0x00, 0x00, 0x28, b'e', b's', b'd', b's', // esds box, size 40
        0x00, 0x00, 0x00, 0x00, // version + flags
        0x03, 0x1a, 0x00, 0x01, 0x00, // EsDescriptor: size 26, es_id 1, flags 0
        0x04, 0x11, 0x40, 0x15, // DecoderConfig: size 17, oti 0x40, stream/up 0x15
        0x00, 0x00, 0x00, // buffer_size_db
        0x00, 0x00, 0x00, 0x00, // max_bitrate
        0x00, 0x00, 0x00, 0x00, // avg_bitrate
        0x05, 0x02, 0x11, 0x90, // DecoderSpecific: size 2 (profile 2, freq 3, chan 2)
        0x06, 0x02, 0x02, 0x15, // SLConfig: size 2 = predefined 0x02 + a trailing byte
    ];

    #[test]
    fn test_esds_two_byte_slconfig() {
        let esds =
            Esds::decode(&mut &ESDS_2BYTE_SLCONFIG[..]).expect("2-byte SLConfig must be tolerated");
        let dec = esds
            .es_desc
            .dec_config
            .dec_specific
            .expect("DecoderSpecificInfo present");
        assert_eq!(dec.profile, 2, "AAC-LC");
        assert_eq!(dec.freq_index, 3, "48 kHz");
    }

    // A `DecoderConfigDescriptor` (tag 0x04) with NO `DecoderSpecificInfo`
    // (tag 0x05) child — valid per ISO/IEC 14496-1, where the DecSpecificInfo
    // is optional (e.g. AAC whose config is carried in-band via ADTS headers).
    // Before the fix the mandatory `MissingDescriptor(0x05)` rejected the whole
    // `esds`; now the field decodes to `None`. Descriptor tree: EsDescriptor
    // (0x03) → DecoderConfig(0x04, no tag-5 child) + SLConfig(0x06).
    const ESDS_NO_DEC_SPECIFIC: &[u8] = &[
        0x00, 0x00, 0x00, 0x23, b'e', b's', b'd', b's', // esds box, size 35
        0x00, 0x00, 0x00, 0x00, // version + flags
        0x03, 0x15, 0x00, 0x01, 0x00, // EsDescriptor: size 21, es_id 1, flags 0
        0x04, 0x0d, 0x40, 0x15, // DecoderConfig: size 13, oti 0x40 (AAC), stream/up 0x15
        0x00, 0x00, 0x00, // buffer_size_db
        0x00, 0x00, 0x00, 0x00, // max_bitrate
        0x00, 0x00, 0x00, 0x00, // avg_bitrate
        0x06, 0x01, 0x02, // SLConfig: size 1, predefined 0x02
    ];

    #[test]
    fn test_esds_missing_dec_specific() {
        let esds = Esds::decode(&mut &ESDS_NO_DEC_SPECIFIC[..])
            .expect("a DecoderConfig without a DecoderSpecificInfo must be tolerated");
        assert_eq!(esds.es_desc.dec_config.object_type_indication, 0x40);
        assert!(
            esds.es_desc.dec_config.dec_specific.is_none(),
            "no tag-5 child decodes to None, not an error"
        );

        // And it round-trips: a `None` dec_specific emits no tag-5 descriptor.
        let mut buf = Vec::new();
        esds.encode(&mut buf).unwrap();
        let again = Esds::decode(&mut buf.as_slice()).unwrap();
        assert_eq!(again, esds);
    }
}
