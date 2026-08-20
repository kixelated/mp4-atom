use crate::*;

pub(crate) const DATA_4CC: FourCC = FourCC::new(b"data");

// Well-known `data` atom type indicators.
// See Apple's [well-known types](https://developer.apple.com/documentation/quicktime-file-format/well-known_types) table.
const TYPE_INDICATOR_RESERVED: u32 = 0u32;
const TYPE_INDICATOR_UTF8: u32 = 1u32;
const TYPE_INDICATOR_UTF16: u32 = 2u32;
const TYPE_INDICATOR_JPEG: u32 = 13u32;
const TYPE_INDICATOR_PNG: u32 = 14u32;
const TYPE_INDICATOR_BE_SIGNED_INT: u32 = 21u32;
const TYPE_INDICATOR_BE_UNSIGNED_INT: u32 = 22u32;
const TYPE_INDICATOR_BE_FLOAT32: u32 = 23u32;
const TYPE_INDICATOR_BE_FLOAT64: u32 = 24u32;
const TYPE_INDICATOR_BMP: u32 = 27u32;
// Fixed-width signed/unsigned integer type indicators, as used by mp4v2 and
// AtomicParsley alongside the variable-length pair above (21/22). Only used
// on decode — `IlstDataValue::to_raw` always canonicalizes to 21/22 on encode.
const TYPE_INDICATOR_SIGNED_INT_FIXED: [u32; 4] = [65, 66, 67, 74];
const TYPE_INDICATOR_UNSIGNED_INT_FIXED: [u32; 4] = [75, 76, 77, 78];

/// A [`IlstData`] value, interpreted according to its wire-level type indicator.
///
/// [`IlstDataValue::Binary`] is the explicit "no type" reserved indicator (`0`).
/// [`IlstDataValue::Unknown`] covers everything else this crate can't interpret:
/// a type indicator it doesn't recognize, or one it does recognize but whose
/// raw bytes don't match (e.g. a `BeFloat32` value that isn't exactly 4
/// bytes) — it carries the original type indicator so callers can still
/// inspect it, and round-trips it unchanged on encode.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IlstDataValue {
    Reserved(Vec<u8>),
    Utf8(String),
    Utf16(String),
    Jpeg(Vec<u8>),
    Png(Vec<u8>),
    Bmp(Vec<u8>),
    BeSignedInt(i64),
    BeUnsignedInt(u64),
    BeFloat32(f32),
    BeFloat64(f64),
    Unknown(u32, Vec<u8>),
}

impl IlstDataValue {
    fn from_raw(type_indicator: u32, value: &[u8]) -> Self {
        match type_indicator {
            TYPE_INDICATOR_RESERVED => IlstDataValue::Reserved(value.to_vec()),
            TYPE_INDICATOR_UTF8 => match std::str::from_utf8(value) {
                Ok(text) => IlstDataValue::Utf8(text.to_string()),
                Err(_) => IlstDataValue::Unknown(type_indicator, value.to_vec()),
            },
            TYPE_INDICATOR_UTF16 => match decode_utf16_be(value) {
                Some(text) => IlstDataValue::Utf16(text),
                None => IlstDataValue::Unknown(type_indicator, value.to_vec()),
            },
            TYPE_INDICATOR_JPEG => IlstDataValue::Jpeg(value.to_vec()),
            TYPE_INDICATOR_PNG => IlstDataValue::Png(value.to_vec()),
            TYPE_INDICATOR_BMP => IlstDataValue::Bmp(value.to_vec()),
            TYPE_INDICATOR_BE_SIGNED_INT => match decode_be_signed_int(value) {
                Some(v) => IlstDataValue::BeSignedInt(v),
                None => IlstDataValue::Unknown(type_indicator, value.to_vec()),
            },
            TYPE_INDICATOR_BE_UNSIGNED_INT => match decode_be_unsigned_int(value) {
                Some(v) => IlstDataValue::BeUnsignedInt(v),
                None => IlstDataValue::Unknown(type_indicator, value.to_vec()),
            },
            t if TYPE_INDICATOR_SIGNED_INT_FIXED.contains(&t) => {
                match decode_be_signed_int(value) {
                    Some(v) => IlstDataValue::BeSignedInt(v),
                    None => IlstDataValue::Unknown(type_indicator, value.to_vec()),
                }
            }
            t if TYPE_INDICATOR_UNSIGNED_INT_FIXED.contains(&t) => {
                match decode_be_unsigned_int(value) {
                    Some(v) => IlstDataValue::BeUnsignedInt(v),
                    None => IlstDataValue::Unknown(type_indicator, value.to_vec()),
                }
            }
            TYPE_INDICATOR_BE_FLOAT32 => match <[u8; 4]>::try_from(value) {
                Ok(bytes) => IlstDataValue::BeFloat32(f32::from_be_bytes(bytes)),
                Err(_) => IlstDataValue::Unknown(type_indicator, value.to_vec()),
            },
            TYPE_INDICATOR_BE_FLOAT64 => match <[u8; 8]>::try_from(value) {
                Ok(bytes) => IlstDataValue::BeFloat64(f64::from_be_bytes(bytes)),
                Err(_) => IlstDataValue::Unknown(type_indicator, value.to_vec()),
            },
            _ => IlstDataValue::Unknown(type_indicator, value.to_vec()),
        }
    }

    fn to_raw(&self) -> (u32, Vec<u8>) {
        match self {
            IlstDataValue::Utf8(text) => (TYPE_INDICATOR_UTF8, text.clone().into_bytes()),
            IlstDataValue::Utf16(text) => (TYPE_INDICATOR_UTF16, encode_utf16_be(text)),
            IlstDataValue::Jpeg(bytes) => (TYPE_INDICATOR_JPEG, bytes.clone()),
            IlstDataValue::Png(bytes) => (TYPE_INDICATOR_PNG, bytes.clone()),
            IlstDataValue::Bmp(bytes) => (TYPE_INDICATOR_BMP, bytes.clone()),
            IlstDataValue::BeSignedInt(v) => {
                (TYPE_INDICATOR_BE_SIGNED_INT, encode_be_signed_int(*v))
            }
            IlstDataValue::BeUnsignedInt(v) => {
                (TYPE_INDICATOR_BE_UNSIGNED_INT, encode_be_unsigned_int(*v))
            }
            IlstDataValue::BeFloat32(v) => (TYPE_INDICATOR_BE_FLOAT32, v.to_be_bytes().to_vec()),
            IlstDataValue::BeFloat64(v) => (TYPE_INDICATOR_BE_FLOAT64, v.to_be_bytes().to_vec()),
            IlstDataValue::Reserved(bytes) => (TYPE_INDICATOR_RESERVED, bytes.clone()),
            IlstDataValue::Unknown(type_indicator, bytes) => (*type_indicator, bytes.clone()),
        }
    }
}

// Apple's well-known BE integer type indicators use a variable-length
// encoding (1, 2, 3, 4, or 8 bytes); 3-byte values need sign/zero-extension.
fn decode_be_signed_int(bytes: &[u8]) -> Option<i64> {
    Some(match bytes.len() {
        1 => i64::from(bytes[0] as i8),
        2 => i64::from(i16::from_be_bytes(bytes.try_into().unwrap())),
        3 => {
            let mut widened = [0u8; 4];
            widened[1..].copy_from_slice(bytes);
            i64::from((i32::from_be_bytes(widened) << 8) >> 8)
        }
        4 => i64::from(i32::from_be_bytes(bytes.try_into().unwrap())),
        8 => i64::from_be_bytes(bytes.try_into().unwrap()),
        _ => return None,
    })
}

fn decode_be_unsigned_int(bytes: &[u8]) -> Option<u64> {
    Some(match bytes.len() {
        1 => u64::from(bytes[0]),
        2 => u64::from(u16::from_be_bytes(bytes.try_into().unwrap())),
        3 => {
            let mut widened = [0u8; 4];
            widened[1..].copy_from_slice(bytes);
            u64::from(u32::from_be_bytes(widened))
        }
        4 => u64::from(u32::from_be_bytes(bytes.try_into().unwrap())),
        8 => u64::from_be_bytes(bytes.try_into().unwrap()),
        _ => return None,
    })
}

// Encode using the narrowest of the widths `decode_be_signed_int` accepts
// (1/2/4/8 bytes; the 3-byte width is decode-only, for compatibility).
fn encode_be_signed_int(v: i64) -> Vec<u8> {
    if let Ok(v) = i8::try_from(v) {
        v.to_be_bytes().to_vec()
    } else if let Ok(v) = i16::try_from(v) {
        v.to_be_bytes().to_vec()
    } else if let Ok(v) = i32::try_from(v) {
        v.to_be_bytes().to_vec()
    } else {
        v.to_be_bytes().to_vec()
    }
}

fn encode_be_unsigned_int(v: u64) -> Vec<u8> {
    if let Ok(v) = u8::try_from(v) {
        v.to_be_bytes().to_vec()
    } else if let Ok(v) = u16::try_from(v) {
        v.to_be_bytes().to_vec()
    } else if let Ok(v) = u32::try_from(v) {
        v.to_be_bytes().to_vec()
    } else {
        v.to_be_bytes().to_vec()
    }
}

fn decode_utf16_be(bytes: &[u8]) -> Option<String> {
    if !bytes.len().is_multiple_of(2) {
        return None;
    }
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16(&units).ok()
}

fn encode_utf16_be(text: &str) -> Vec<u8> {
    text.encode_utf16().flat_map(u16::to_be_bytes).collect()
}

/// The content of an iTunes-style `data` atom.
///
/// Unlike the well-known `ilst` tags ([`Copyright`], [`Tool`], [`Desc`], ...),
/// which are always UTF-8 text, Apple's `mdta`-keyed metadata items (see
/// [`Keys`]) can hold any of the well-known `data` type indicators — `value`
/// is a [`IlstDataValue`] that interprets the wire-level type indicator so
/// callers don't have to.
///
/// Two encodings exist in the wild: the FFmpeg "short" style, where the value
/// follows the item header directly, and the QuickTime/GPAC "long" style,
/// where the value is wrapped in a nested `data` atom. Both decode into this
/// same representation; encoding always emits the long style.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IlstData {
    pub country_indicator: u16,
    pub language_indicator: u16,
    pub value: IlstDataValue,
}

impl IlstData {
    pub(crate) fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        let type_indicator_or_len = u32::decode(buf)?;

        let type_indicator = match type_indicator_or_len {
            TYPE_INDICATOR_UTF8 => {
                // Too short for a valid length, so probably
                // UTF-8 text, FFmpeg short-style
                type_indicator_or_len
            }
            _ => {
                // Maybe Atom follows on straight away.
                // Try parsing as Quicktime data atom: GPAC style or FFmpeg long style
                let fourcc = FourCC::decode(buf)?;
                if fourcc != DATA_4CC {
                    return Err(Error::UnexpectedBox(fourcc));
                }

                u32::decode(buf)?
            }
        };

        let country_indicator = u16::decode(buf)?;
        let language_indicator = u16::decode(buf)?;

        let size = buf.remaining();
        let value = IlstDataValue::from_raw(type_indicator, buf.slice(size));
        buf.advance(size);

        Ok(Self {
            country_indicator,
            language_indicator,
            value,
        })
    }

    pub(crate) fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let (type_indicator, raw) = self.value.to_raw();

        // the length of the nested atom is the length field (4 bytes),
        // the 4CC (4 bytes), the type indicator (4 bytes), the country
        // indicator (2 bytes), the language indicator (2 bytes) and
        // then the actual value.
        let nested_len = (4 + 4 + 4 + 2 + 2 + raw.len()) as u32;
        nested_len.encode(buf)?;
        DATA_4CC.encode(buf)?;
        type_indicator.encode(buf)?;
        self.country_indicator.encode(buf)?;
        self.language_indicator.encode(buf)?;
        raw.as_slice().encode(buf)?;
        Ok(())
    }
}

/// A UTF-8 text payload of an iTunes-style `ilst` metadata item.
pub(crate) struct DataText {
    pub country_indicator: u16,
    pub language_indicator: u16,
    pub text: String,
}

pub(crate) fn decode_text<B: Buf>(buf: &mut B) -> Result<DataText> {
    let data = IlstData::decode(buf)?;
    match data.value {
        IlstDataValue::Utf8(text) => Ok(DataText {
            country_indicator: data.country_indicator,
            language_indicator: data.language_indicator,
            text,
        }),
        _ => Err(Error::Unsupported(
            "Only UTF-8 text is supported in ilst data atoms",
        )),
    }
}

pub(crate) fn encode_text<B: BufMut>(
    country_indicator: u16,
    language_indicator: u16,
    text: &str,
    buf: &mut B,
) -> Result<()> {
    IlstData {
        country_indicator,
        language_indicator,
        value: IlstDataValue::Utf8(text.to_string()),
    }
    .encode(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Long QuickTime/GPAC style: the value is wrapped in a nested `data` atom.
    // This is the layout `encode_text` always emits.
    const LONG_STYLE: &[u8] = &[
        0x00, 0x00, 0x00, 0x1A, // data atom size = 26
        b'd', b'a', b't', b'a', //
        0x00, 0x00, 0x00, 0x01, // type indicator: UTF-8
        0x00, 0x00, 0x00, 0x00, // country + language
        b'(', b'c', b')', b' ', b'2', b'0', b'2', b'6', b' ', b'x',
    ];

    #[test]
    fn test_decode_text_short_style() {
        // FFmpeg short style: the leading `1` is the type indicator (too small
        // to be a valid `data` atom length), so the UTF-8 value follows directly.
        let buf = [
            0x00, 0x00, 0x00, 0x01, // type indicator (short style): UTF-8
            0x00, 0x00, 0x00, 0x00, // country + language
            b'2', b'0', b'2', b'6',
        ];
        let decoded = decode_text(&mut &buf[..]).unwrap();
        assert_eq!(decoded.country_indicator, 0);
        assert_eq!(decoded.language_indicator, 0);
        assert_eq!(decoded.text, "2026");
    }

    #[test]
    fn test_decode_text_long_style() {
        let decoded = decode_text(&mut &LONG_STYLE[..]).unwrap();
        assert_eq!(decoded.country_indicator, 0);
        assert_eq!(decoded.language_indicator, 0);
        assert_eq!(decoded.text, "(c) 2026 x");
    }

    #[test]
    fn test_encode_text_emits_long_style() {
        // `encode_text` always emits the long `data`-wrapped layout, which
        // decodes back to the same value.
        let mut buf = Vec::new();
        encode_text(0, 0, "(c) 2026 x", &mut buf).unwrap();
        assert_eq!(buf, LONG_STYLE);

        let decoded = decode_text(&mut buf.as_slice()).unwrap();
        assert_eq!(decoded.text, "(c) 2026 x");
    }

    fn data_with(value: IlstDataValue) -> IlstData {
        IlstData {
            country_indicator: 0,
            language_indicator: 0,
            value,
        }
    }

    #[test]
    fn test_data_roundtrip_non_utf8_type() {
        // `mdta`-keyed items can use type indicators other than UTF-8 text.
        let data = data_with(IlstDataValue::BeSignedInt(42));

        let mut buf = Vec::new();
        data.encode(&mut buf).unwrap();

        let decoded = IlstData::decode(&mut buf.as_slice()).expect("failed to decode data");
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_data_roundtrip_unknown_preserves_exact_bytes() {
        // `Unknown` carries the original type indicator, so it round-trips
        // byte-for-byte even though this crate doesn't otherwise recognize it.
        let data = data_with(IlstDataValue::Unknown(999, vec![1, 2, 3]));

        let mut buf = Vec::new();
        data.encode(&mut buf).unwrap();

        let decoded = IlstData::decode(&mut buf.as_slice()).expect("failed to decode data");
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_decode_value_utf8() {
        let mut data_body = Vec::new();
        data_body.extend_from_slice(&1u32.to_be_bytes());
        data_body.extend_from_slice(&0u32.to_be_bytes());
        data_body.extend_from_slice(b"hello");

        let decoded = IlstData::decode(&mut data_body.as_slice()).unwrap();
        assert_eq!(decoded.value, IlstDataValue::Utf8("hello".into()));
    }

    #[test]
    fn test_decode_value_utf16() {
        let text = "hi \u{263A}"; // includes a non-ASCII code point
        assert_eq!(
            decode_raw(2, &encode_utf16_be(text)),
            IlstDataValue::Utf16(text.into())
        );
    }

    #[test]
    fn test_decode_value_jpeg_png_bmp() {
        assert_eq!(
            decode_raw(13, &[0xFF, 0xD8]),
            IlstDataValue::Jpeg(vec![0xFF, 0xD8])
        );
        assert_eq!(
            decode_raw(14, &[0x89, 0x50]),
            IlstDataValue::Png(vec![0x89, 0x50])
        );
        assert_eq!(
            decode_raw(27, &[0x42, 0x4D]),
            IlstDataValue::Bmp(vec![0x42, 0x4D])
        );
    }

    // Build the long-style nested `data` atom layout `IlstData::decode` expects
    // for any type indicator other than the short-style `1`.
    fn decode_raw(type_indicator: u32, value: &[u8]) -> IlstDataValue {
        let mut data_body = Vec::new();
        let nested_len = (4 + 4 + 4 + 2 + 2 + value.len()) as u32;
        data_body.extend_from_slice(&nested_len.to_be_bytes());
        data_body.extend_from_slice(b"data");
        data_body.extend_from_slice(&type_indicator.to_be_bytes());
        data_body.extend_from_slice(&0u32.to_be_bytes()); // country + language
        data_body.extend_from_slice(value);
        IlstData::decode(&mut data_body.as_slice()).unwrap().value
    }

    #[test]
    fn test_decode_value_be_signed_int_variable_length() {
        assert_eq!(decode_raw(21, &[0xFF]), IlstDataValue::BeSignedInt(-1));
        assert_eq!(
            decode_raw(21, &1000i16.to_be_bytes()),
            IlstDataValue::BeSignedInt(1000)
        );
        assert_eq!(
            decode_raw(21, &(-1i32).to_be_bytes()[1..]),
            IlstDataValue::BeSignedInt(-1)
        );
        assert_eq!(
            decode_raw(21, &(-100000i32).to_be_bytes()),
            IlstDataValue::BeSignedInt(-100000)
        );
        assert_eq!(
            decode_raw(21, &(-1i64).to_be_bytes()),
            IlstDataValue::BeSignedInt(-1)
        );
    }

    #[test]
    fn test_decode_value_be_unsigned_int_variable_length() {
        assert_eq!(decode_raw(22, &[200]), IlstDataValue::BeUnsignedInt(200));
        assert_eq!(
            decode_raw(22, &40000u32.to_be_bytes()),
            IlstDataValue::BeUnsignedInt(40000)
        );
    }

    #[test]
    fn test_decode_value_fixed_width_ints() {
        // mp4v2/AtomicParsley-style fixed-width type indicators, distinct
        // from the variable-length 21/22 pair.
        assert_eq!(
            decode_raw(67, &42i32.to_be_bytes()),
            IlstDataValue::BeSignedInt(42)
        );
        assert_eq!(
            decode_raw(77, &42u32.to_be_bytes()),
            IlstDataValue::BeUnsignedInt(42)
        );
    }

    #[test]
    fn test_decode_value_be_float32_and_float64() {
        assert_eq!(
            decode_raw(23, &1.5f32.to_be_bytes()),
            IlstDataValue::BeFloat32(1.5)
        );
        assert_eq!(
            decode_raw(24, &1.5f64.to_be_bytes()),
            IlstDataValue::BeFloat64(1.5)
        );
    }

    #[test]
    fn test_decode_value_binary_for_reserved_type() {
        assert_eq!(
            decode_raw(0, &[1, 2, 3]),
            IlstDataValue::Reserved(vec![1, 2, 3])
        );
    }

    #[test]
    fn test_decode_value_unknown_for_unrecognized_or_malformed() {
        // Unrecognized type indicator.
        assert_eq!(
            decode_raw(999, &[1, 2, 3]),
            IlstDataValue::Unknown(999, vec![1, 2, 3])
        );
        // Wrong length for the declared type.
        assert_eq!(
            decode_raw(23, &[1, 2, 3]),
            IlstDataValue::Unknown(23, vec![1, 2, 3])
        );
    }
}
