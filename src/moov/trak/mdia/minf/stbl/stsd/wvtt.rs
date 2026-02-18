use crate::*;

// WebVTTConfigurationBox
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VttC {
    pub config: String,
}

impl Atom for VttC {
    const KIND: FourCC = FourCC::new(b"vttC");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Self {
            config: decode_boxstring(buf)?,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.config.as_bytes().encode(buf)
    }
}

// WebVTTSourceLabelBox
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vlab {
    pub source_label: String,
}

impl Atom for Vlab {
    const KIND: FourCC = FourCC::new(b"vlab");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Self {
            source_label: decode_boxstring(buf)?,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        // boxstring is just UTF-8 encoded string
        self.source_label.as_bytes().encode(buf)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Wvtt {
    pub plaintext: PlainText,
    pub config: VttC,
    pub label: Option<Vlab>,
    pub btrt: Option<Btrt>,
}

impl Atom for Wvtt {
    const KIND: FourCC = FourCC::new(b"wvtt");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let plaintext = PlainText::decode(buf)?;

        let mut vtcc = None;
        let mut vlab = None;
        let mut btrt = None;

        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::VttC(atom) => vtcc = atom.into(),
                Any::Vlab(atom) => vlab = atom.into(),
                Any::Btrt(atom) => btrt = atom.into(),
                unknown => Self::decode_unknown(&unknown)?,
            }
        }

        Ok(Self {
            plaintext,
            config: vtcc.ok_or(Error::MissingBox(VttC::KIND))?,
            label: vlab,
            btrt,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.plaintext.encode(buf)?;
        self.config.encode(buf)?;
        self.label.encode(buf)?;
        self.btrt.encode(buf)?;
        Ok(())
    }
}

fn decode_boxstring<B: Buf>(buf: &mut B) -> Result<String> {
    let remaining_bytes = buf.remaining();
    let body = &mut buf.slice(remaining_bytes);
    let text = String::from_utf8(body.to_vec()).map_err(|_| Error::InvalidSize)?;
    buf.advance(remaining_bytes);
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    // from MPEG File Format Conformance suite, 16_vtt.mp4
    const ENCODED_WVTT: &[u8] = &[
        0x00, 0x00, 0x00, 0x3d, 0x77, 0x76, 0x74, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x2d, 0x76, 0x74, 0x74, 0x43, 0x57, 0x45, 0x42, 0x56, 0x54, 0x54,
        0x0a, 0x53, 0x6f, 0x6d, 0x65, 0x20, 0x64, 0x75, 0x6d, 0x6d, 0x79, 0x20, 0x68, 0x65, 0x61,
        0x64, 0x65, 0x72, 0x20, 0x69, 0x6e, 0x66, 0x6f, 0x72, 0x6d, 0x61, 0x74, 0x69, 0x6f, 0x6e,
        0x0a,
    ];

    #[test]
    fn test_wvtt_decode() {
        let buf: &mut std::io::Cursor<&[u8]> = &mut std::io::Cursor::new(ENCODED_WVTT);

        let wvtt = Wvtt::decode(buf).expect("failed to decode wvtt");

        assert_eq!(
            wvtt,
            Wvtt {
                plaintext: PlainText {
                    data_reference_index: 1
                },
                config: VttC {
                    config: "WEBVTT\nSome dummy header information\n".into()
                },
                label: None,
                btrt: None,
            }
        );
    }

    #[test]
    fn test_wvtt_encode() {
        let wvtt = Wvtt {
            plaintext: PlainText {
                data_reference_index: 1,
            },
            config: VttC {
                config: "WEBVTT\nSome dummy header information\n".into(),
            },
            label: None,
            btrt: None,
        };

        let mut buf = Vec::new();
        wvtt.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_WVTT);
    }

    #[test]
    fn test_round_trip_with_label() {
        let wvtt = Wvtt {
            plaintext: PlainText {
                data_reference_index: 1,
            },
            config: VttC {
                config: "WEBVTT\nSome dummy header information\n".into(),
            },
            label: Some(Vlab {
                source_label: "uri://dummy/label".into(),
            }),
            btrt: Some(Btrt {
                buffer_size_db: 1,
                max_bitrate: 2000,
                avg_bitrate: 400,
            }),
        };

        let mut buf = Vec::new();
        wvtt.encode(&mut buf).unwrap();
        let decoded = Wvtt::decode(&mut buf.as_ref()).expect("failed to decode wvtt");

        assert_eq!(decoded, wvtt,);
    }
}
