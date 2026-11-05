use std::ops::Range;

use crate::*;

const STYL: FourCC = FourCC::new(b"styl");
const HLIT: FourCC = FourCC::new(b"hlit");
const HCLR: FourCC = FourCC::new(b"hclr");
const DLAY: FourCC = FourCC::new(b"dlay");
const TBOX: FourCC = FourCC::new(b"tbox");
const BLNK: FourCC = FourCC::new(b"blnk");
const TWRP: FourCC = FourCC::new(b"twrp");
const KROK: FourCC = FourCC::new(b"krok");
const HREF: FourCC = FourCC::new(b"href");

/// A `StyleRecord`, see 3GPP TS 26.245 Section 5.16.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tx3gModifierStylRecord {
    /// The characters of the text the style applies to, as offsets in
    /// characters; the end is the first character the style doesn't apply to.
    pub range: Range<u16>,
    pub font_id: u16,
    pub face_style_flags: u8,
    pub font_size: u8,
    pub text_color: RgbaColor,
}

/// One highlight event of a [`Tx3gModifier::Krok`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tx3gModifierKrokEvent {
    pub end_time: u32,
    pub range: Range<u16>,
}

/// A text modifier box following the text of a [`Tx3gSample`].
///
/// See 3GPP TS 26.245 Section 5.17.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Tx3gModifier {
    /// `TextStyleBox` ('styl')
    Styl(Vec<Tx3gModifierStylRecord>),
    /// `TextHighlightBox` ('hlit')
    Hlit(Range<u16>),
    /// `TextHighlightColorBox` ('hclr')
    Hclr(RgbaColor),
    /// `TextDelayBox` ('dlay'), the scroll delay in timescale units.
    Dlay(u32),
    /// `TextBoxBox` ('tbox'): top, left, bottom, right.
    Tbox([i16; 4]),
    /// `TextBlinkBox` ('blnk')
    Blnk(Range<u16>),
    /// `TextWrapBox` ('twrp')
    Twrp(u8),
    /// `TextKaraokeBox` ('krok'): dynamic highlighting. Times are in track
    /// timescale units, relative to the start of the sample.
    Krok {
        highlight_start_time: u32,
        events: Vec<Tx3gModifierKrokEvent>,
    },
    /// `TextHyperTextBox` ('href'): a hyperlink on a range of the text.
    Href {
        range: Range<u16>,
        url: String,
        alt: String,
    },
    /// `DisparityBox` ('disp'): overrides the entry's default disparity.
    Disp(Disp),
    /// Any other modifier; the raw body is preserved.
    Unknown(FourCC, Vec<u8>),
}

/// The content of a single sample in a 3GPP timed text track (the [`Tx3g`]
/// sample entry): the text followed by optional modifier boxes.
///
/// Sample content lives inside `mdat` and isn't walked automatically by this
/// crate -- decode it explicitly via [`Tx3gSample::decode`].
///
/// The text is UTF-8 or, if it starts with a byte order mark, UTF-16. Both
/// decode to a [`String`]; encoding always writes UTF-8.
///
/// See 3GPP TS 26.245 Section 5.16.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tx3gSample {
    pub text: String,
    pub modifiers: Vec<Tx3gModifier>,
}

impl Tx3gModifier {
    fn decode_body<B: Buf>(kind: FourCC, buf: &mut B) -> Result<Self> {
        Ok(match kind {
            STYL => Self::Styl(buf.decode_list(1024u16, |buf| {
                Ok(Tx3gModifierStylRecord {
                    range: Range::decode(buf)?,
                    font_id: u16::decode(buf)?,
                    face_style_flags: u8::decode(buf)?,
                    font_size: u8::decode(buf)?,
                    text_color: RgbaColor::decode(buf)?,
                })
            })?),
            HLIT => Self::Hlit(Range::decode(buf)?),
            HCLR => Self::Hclr(RgbaColor::decode(buf)?),
            DLAY => Self::Dlay(u32::decode(buf)?),
            TBOX => Self::Tbox([
                i16::decode(buf)?,
                i16::decode(buf)?,
                i16::decode(buf)?,
                i16::decode(buf)?,
            ]),
            BLNK => Self::Blnk(Range::decode(buf)?),
            TWRP => Self::Twrp(u8::decode(buf)?),
            KROK => Self::Krok {
                highlight_start_time: u32::decode(buf)?,
                events: buf.decode_list(1024u16, |buf| {
                    Ok(Tx3gModifierKrokEvent {
                        end_time: u32::decode(buf)?,
                        range: Range::decode(buf)?,
                    })
                })?,
            },
            HREF => Self::Href {
                range: Range::decode(buf)?,
                url: decode_length_prefixed_utf8_string::<u8>(buf)?,
                alt: decode_length_prefixed_utf8_string::<u8>(buf)?,
            },
            Disp::KIND => Self::Disp(Disp::decode_body(buf)?),
            kind => {
                let remaining = buf.remaining();
                let body = buf.slice(remaining).to_vec();
                buf.advance(remaining);
                Self::Unknown(kind, body)
            }
        })
    }

    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        match self {
            Self::Styl(records) => buf.encode_atom(STYL, |buf| {
                buf.encode_list::<u16, _>(records, STYL, |buf, r| {
                    r.range.encode(buf)?;
                    r.font_id.encode(buf)?;
                    r.face_style_flags.encode(buf)?;
                    r.font_size.encode(buf)?;
                    r.text_color.encode(buf)
                })
            }),
            Self::Hlit(range) => buf.encode_atom_value(HLIT, range),
            Self::Hclr(color) => buf.encode_atom_value(HCLR, color),
            Self::Dlay(delay) => buf.encode_atom_value(DLAY, delay),
            Self::Tbox(rect) => buf.encode_atom(TBOX, |buf| {
                for v in rect {
                    v.encode(buf)?;
                }
                Ok(())
            }),
            Self::Blnk(range) => buf.encode_atom_value(BLNK, range),
            Self::Twrp(wrap) => buf.encode_atom_value(TWRP, wrap),
            Self::Krok {
                highlight_start_time,
                events,
            } => buf.encode_atom(KROK, |buf| {
                highlight_start_time.encode(buf)?;
                buf.encode_list::<u16, _>(events, KROK, |buf, e| {
                    e.end_time.encode(buf)?;
                    e.range.encode(buf)
                })
            }),
            Self::Href { range, url, alt } => buf.encode_atom(HREF, |buf| {
                range.encode(buf)?;
                encode_length_prefixed_string::<u8>(buf, url, HREF)?;
                encode_length_prefixed_string::<u8>(buf, alt, HREF)
            }),
            Self::Disp(disp) => disp.encode(buf),
            Self::Unknown(kind, body) => buf.encode_atom(*kind, |buf| {
                buf.append_slice(body);
                Ok(())
            }),
        }
    }
}

impl Tx3gSample {
    pub fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        let text = decode_length_prefixed_bom_string::<u16>(buf)?;

        let mut modifiers = Vec::new();
        while let Some(header) = Header::decode_maybe(buf)? {
            modifiers.push(
                header.decode_body(buf, |body| Tx3gModifier::decode_body(header.kind, body))?,
            );
        }

        Ok(Self { text, modifiers })
    }

    pub fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        encode_length_prefixed_string::<u16>(buf, &self.text, Tx3g::KIND)?;

        for modifier in &self.modifiers {
            modifier.encode(buf)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx3g_sample_roundtrip() {
        let sample = Tx3gSample {
            text: "Hello, wörld".into(),
            modifiers: vec![
                Tx3gModifier::Styl(vec![Tx3gModifierStylRecord {
                    range: 0..5,
                    font_id: 1,
                    face_style_flags: 1,
                    font_size: 18,
                    text_color: RgbaColor {
                        red: 255,
                        green: 255,
                        blue: 255,
                        alpha: 255,
                    },
                }]),
                Tx3gModifier::Hlit(1..3),
                Tx3gModifier::Dlay(500),
                Tx3gModifier::Tbox([0, 0, 10, 10]),
                Tx3gModifier::Twrp(1),
                Tx3gModifier::Krok {
                    highlight_start_time: 10,
                    events: vec![
                        Tx3gModifierKrokEvent {
                            end_time: 100,
                            range: 0..3,
                        },
                        Tx3gModifierKrokEvent {
                            end_time: 200,
                            range: 3..3,
                        },
                    ],
                },
                Tx3gModifier::Href {
                    range: 0..5,
                    url: "https://example.com".into(),
                    alt: "Example".into(),
                },
                Tx3gModifier::Disp(Disp {
                    disparity_shift_in_16th_pel: -32,
                }),
                Tx3gModifier::Unknown(FourCC::new(b"xxxx"), vec![1, 2, 3]),
            ],
        };

        let mut buf = Vec::new();
        sample.encode(&mut buf).unwrap();
        let decoded = Tx3gSample::decode(&mut buf.as_slice()).expect("failed to decode sample");
        assert_eq!(decoded, sample);
    }

    #[test]
    fn test_tx3g_sample_empty() {
        let mut buf = Vec::new();
        Tx3gSample::default().encode(&mut buf).unwrap();
        assert_eq!(buf, [0, 0]);
        assert_eq!(
            Tx3gSample::decode(&mut buf.as_slice()).unwrap(),
            Tx3gSample::default()
        );
    }

    #[test]
    fn test_tx3g_sample_truncated_text() {
        let bytes = [0, 10, b'a'];
        let err = Tx3gSample::decode(&mut bytes.as_slice()).unwrap_err();
        assert!(matches!(err, Error::OutOfBounds));
    }
}
