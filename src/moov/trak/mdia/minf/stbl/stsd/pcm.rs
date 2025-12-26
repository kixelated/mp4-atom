use crate::*;

ext! {
    name: PcmC,
    versions: [0],
    flags: {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PcmC {
    pub big_endian: bool,
    pub sample_size: u8,
}

impl AtomExt for PcmC {
    const KIND_EXT: FourCC = FourCC::new(b"pcmC");

    type Ext = PcmCExt;

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: PcmCExt) -> Result<Self> {
        let format_flags = u8::decode(buf)?;
        let sample_size = u8::decode(buf)?;

        Ok(Self {
            big_endian: format_flags == 0,
            sample_size,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<PcmCExt> {
        let mut format_flags = 0u8;
        if !self.big_endian {
            format_flags = 1u8;
        }

        format_flags.encode(buf)?;
        self.sample_size.encode(buf)?;

        Ok(PcmCExt {
            version: PcmCVersion::V0,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pcm {
    pub fourcc: FourCC,
    pub audio: Audio,
    pub pcmc: PcmC,
    pub chnl: Option<Chnl>,
    pub btrt: Option<Btrt>,
}

impl Pcm {
    fn encode_fields<B: BufMut>(
        audio: &Audio,
        pcmc: &PcmC,
        chnl: Option<&Chnl>,
        btrt: Option<&Btrt>,
        buf: &mut B,
    ) -> Result<()> {
        audio.encode(buf)?;
        pcmc.encode(buf)?;

        if let Some(chnl) = chnl {
            chnl.encode(buf)?;
        }

        if let Some(btrt) = btrt {
            btrt.encode(buf)?;
        }

        Ok(())
    }

    pub fn decode_with_fourcc<B: Buf>(fourcc: FourCC, buf: &mut B) -> Result<Self> {
        let audio = Audio::decode(buf)?;

        let mut chnl = None;
        let mut pcmc = None;
        let mut btrt = None;

        while buf.remaining() > 0 {
            let header = match Header::decode_maybe(buf)? {
                Some(h) => h,
                None => break,
            };

            let size = header.size.unwrap_or(buf.remaining());
            if size > buf.remaining() {
                break;
            }

            let mut limited = buf.slice(size);

            if header.kind == Chnl::KIND {
                // Decode channel layout by using the channel count
                // information. We cannot rely on the decode_body
                // implementation of Atom for channel layout box.
                chnl = Some(Chnl::decode_body_with_channel_count(
                    &mut limited,
                    audio.channel_count,
                )?);
            } else {
                match Any::decode_atom(&header, &mut limited)? {
                    Any::PcmC(atom) => pcmc = Some(atom),
                    Any::Btrt(atom) => btrt = Some(atom),
                    _ => tracing::warn!("unknown atom in PCM sample entry: {:?}", header.kind),
                }
            }

            buf.advance(size);
        }

        Ok(Self {
            fourcc,
            audio,
            pcmc: pcmc.ok_or(Error::MissingBox(PcmC::KIND))?,
            chnl,
            btrt,
        })
    }

    pub fn encode_with_fourcc<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        Self::encode_fields(
            &self.audio,
            &self.pcmc,
            self.chnl.as_ref(),
            self.btrt.as_ref(),
            buf,
        )
    }
}

macro_rules! define_pcm_sample_entry {
    ($name:ident, $fourcc:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct $name {
            pub audio: Audio,
            pub pcmc: PcmC,
            pub chnl: Option<Chnl>,
            pub btrt: Option<Btrt>,
        }

        impl Atom for $name {
            const KIND: FourCC = FourCC::new($fourcc);

            fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
                let entry = Pcm::decode_with_fourcc(Self::KIND, buf)?;
                Ok(Self {
                    audio: entry.audio,
                    pcmc: entry.pcmc,
                    chnl: entry.chnl,
                    btrt: entry.btrt,
                })
            }

            fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
                Pcm::encode_fields(
                    &self.audio,
                    &self.pcmc,
                    self.chnl.as_ref(),
                    self.btrt.as_ref(),
                    buf,
                )
            }
        }
    };
}

define_pcm_sample_entry!(Sowt, b"sowt");
define_pcm_sample_entry!(Twos, b"twos");
define_pcm_sample_entry!(Lpcm, b"lpcm");
define_pcm_sample_entry!(Ipcm, b"ipcm");
define_pcm_sample_entry!(Fpcm, b"fpcm");
define_pcm_sample_entry!(In24, b"in24");
define_pcm_sample_entry!(In32, b"in32");
define_pcm_sample_entry!(Fl32, b"fl32");
define_pcm_sample_entry!(Fl64, b"fl64");
define_pcm_sample_entry!(S16l, b"s16l");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pcmc_encode_decode() {
        let pcmc = PcmC {
            big_endian: true,
            sample_size: 16,
        };

        let mut buf = Vec::new();
        pcmc.encode(&mut buf).unwrap();

        let decoded = PcmC::decode(&mut &buf[..]).unwrap();
        assert_eq!(pcmc, decoded);
    }

    #[test]
    fn test_pcm_encode_decode() {
        let pcmc = PcmC {
            big_endian: false,
            sample_size: 16,
        };
        let chnl = Chnl {
            channel_structure: Some(ChannelStructure::DefinedLayout {
                layout: 2, // Stereo
                omitted_channels_map: Some(0),
                channel_order_definition: None,
            }),
            object_count: None,
            format_ordering: None,
            base_channel_count: None,
        };
        let pcm = Pcm {
            fourcc: FourCC::new(b"fpcm"),
            audio: Audio {
                data_reference_index: 1,
                channel_count: 2,
                sample_size: 16,
                sample_rate: 48000.into(),
            },
            pcmc,
            chnl: Some(chnl),
            btrt: None,
        };

        let mut buf = Vec::new();
        pcm.encode_with_fourcc(&mut buf).unwrap();

        let decoded = Pcm::decode_with_fourcc(FourCC::new(b"fpcm"), &mut &buf[..]).unwrap();
        assert_eq!(pcm, decoded);
    }

    #[test]
    fn test_pcm_encode_decode_with_both_channel_and_object() {
        let pcmc = PcmC {
            big_endian: true,
            sample_size: 16,
        };
        let chnl = Chnl {
            channel_structure: Some(ChannelStructure::DefinedLayout {
                layout: 2, // Stereo
                omitted_channels_map: Some(0),
                channel_order_definition: None,
            }),
            object_count: Some(2),
            format_ordering: None,
            base_channel_count: None,
        };
        let pcm = Pcm {
            fourcc: FourCC::new(b"ipcm"),
            audio: Audio {
                data_reference_index: 1,
                channel_count: 2,
                sample_size: 16,
                sample_rate: 48000.into(),
            },
            pcmc,
            chnl: Some(chnl),
            btrt: None,
        };

        let mut buf = Vec::new();
        pcm.encode_with_fourcc(&mut buf).unwrap();

        let decoded = Pcm::decode_with_fourcc(FourCC::new(b"ipcm"), &mut &buf[..]).unwrap();
        assert_eq!(pcm, decoded);
    }

    #[test]
    fn test_pcm_encode_decode_with_both_channel_and_object_explicit_position() {
        let pcmc = PcmC {
            big_endian: true,
            sample_size: 16,
        };
        let chnl = Chnl {
            channel_structure: Some(ChannelStructure::ExplicitPositions {
                positions: vec![
                    SpeakerPosition::Standard(AudioChannelPosition::FrontLeft),
                    SpeakerPosition::Standard(AudioChannelPosition::FrontRight),
                ],
            }),
            object_count: Some(2),
            format_ordering: None,
            base_channel_count: None,
        };
        let pcm = Pcm {
            fourcc: FourCC::new(b"fpcm"),
            audio: Audio {
                data_reference_index: 1,
                channel_count: 2,
                sample_size: 16,
                sample_rate: 48000.into(),
            },
            pcmc,
            chnl: Some(chnl),
            btrt: Some(Btrt {
                buffer_size_db: 6,
                max_bitrate: 2_304_096,
                avg_bitrate: 2_304_000,
            }),
        };

        let mut buf = Vec::new();
        pcm.encode_with_fourcc(&mut buf).unwrap();

        let decoded = Pcm::decode_with_fourcc(FourCC::new(b"fpcm"), &mut &buf[..]).unwrap();
        assert_eq!(pcm, decoded);
    }

    #[test]
    fn test_pcm_encode_decode_with_chnl_explicit_positions() {
        let pcmc = PcmC {
            big_endian: true,
            sample_size: 16,
        };
        let chnl = Chnl {
            channel_structure: Some(ChannelStructure::ExplicitPositions {
                positions: vec![
                    SpeakerPosition::Standard(AudioChannelPosition::FrontLeft),
                    SpeakerPosition::Standard(AudioChannelPosition::FrontRight),
                ],
            }),
            object_count: None,
            format_ordering: None,
            base_channel_count: None,
        };
        let pcm = Pcm {
            fourcc: FourCC::new(b"fpcm"),
            audio: Audio {
                data_reference_index: 1,
                channel_count: 2,
                sample_size: 16,
                sample_rate: 48000.into(),
            },
            pcmc,
            chnl: Some(chnl),
            btrt: None,
        };

        let mut buf = Vec::new();
        pcm.encode_with_fourcc(&mut buf).unwrap();

        let decoded = Pcm::decode_with_fourcc(FourCC::new(b"fpcm"), &mut &buf[..]).unwrap();
        assert_eq!(pcm, decoded);
    }

    #[test]
    fn test_pcm_encode_decode_with_chnl_explicit_speaker_position() {
        let pcmc = PcmC {
            big_endian: true,
            sample_size: 16,
        };
        let chnl = Chnl {
            channel_structure: Some(ChannelStructure::ExplicitPositions {
                positions: vec![
                    SpeakerPosition::Standard(AudioChannelPosition::FrontLeft),
                    SpeakerPosition::Explicit(ExplicitSpeakerPosition {
                        azimuth: 45,
                        elevation: 10,
                    }),
                ],
            }),
            object_count: None,
            format_ordering: None,
            base_channel_count: None,
        };
        let pcm = Pcm {
            fourcc: FourCC::new(b"fpcm"),
            audio: Audio {
                data_reference_index: 1,
                channel_count: 2,
                sample_size: 16,
                sample_rate: 48000.into(),
            },
            pcmc,
            chnl: Some(chnl),
            btrt: None,
        };

        let mut buf = Vec::new();
        pcm.encode_with_fourcc(&mut buf).unwrap();

        let decoded = Pcm::decode_with_fourcc(FourCC::new(b"fpcm"), &mut &buf[..]).unwrap();
        assert_eq!(pcm, decoded);
    }

    #[test]
    fn test_pcm_v1_chnl() {
        let pcmc = PcmC {
            big_endian: false,
            sample_size: 24,
        };
        let chnl = Chnl {
            channel_structure: Some(ChannelStructure::ExplicitPositions {
                positions: vec![
                    SpeakerPosition::Standard(AudioChannelPosition::FrontLeft),
                    SpeakerPosition::Standard(AudioChannelPosition::FrontRight),
                ],
            }),
            object_count: Some(1),
            format_ordering: Some(1),
            base_channel_count: Some(3),
        };
        let pcm = Lpcm {
            audio: Audio {
                data_reference_index: 1,
                channel_count: 2,
                sample_size: 24,
                sample_rate: 48000.into(),
            },
            pcmc,
            chnl: Some(chnl),
            btrt: None,
        };

        let mut buf = Vec::new();
        pcm.encode(&mut buf).unwrap();

        let decoded = Lpcm::decode(&mut &buf[..]).unwrap();
        assert_eq!(pcm, decoded);
    }

    const ENCODED_IPCM: &[u8] = &[
        0x00, 0x00, 0x00, 0x5c, 0x69, 0x70, 0x63, 0x6d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x18, 0x00, 0x00,
        0x00, 0x00, 0xbb, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0x70, 0x63, 0x6d, 0x43, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00, 0x16, 0x63, 0x68, 0x6e, 0x6c, 0x00, 0x00,
        0x00, 0x00, 0x01, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x14, 0x62, 0x74, 0x72, 0x74, 0x00, 0x00, 0x00, 0x06, 0x00, 0x23, 0x28, 0x60, 0x00, 0x23,
        0x28, 0x00,
    ];

    fn decoded_ipcm() -> Ipcm {
        Ipcm {
            audio: Audio {
                data_reference_index: 1,
                channel_count: 2,
                sample_size: 24,
                sample_rate: FixedPoint::new(48000, 0),
            },
            pcmc: PcmC {
                big_endian: true,
                sample_size: 24,
            },
            chnl: Some(Chnl {
                channel_structure: Some(ChannelStructure::DefinedLayout {
                    layout: 2,
                    omitted_channels_map: Some(0),
                    channel_order_definition: None,
                }),
                object_count: None,
                format_ordering: None,
                base_channel_count: None,
            }),
            btrt: Some(Btrt {
                buffer_size_db: 6,
                max_bitrate: 2_304_096,
                avg_bitrate: 2_304_000,
            }),
        }
    }

    #[test]
    fn test_ipcm_decode() {
        let buf = &mut std::io::Cursor::new(ENCODED_IPCM);
        let ipcm = Ipcm::decode(buf).expect("failed to decode ipcm");
        assert_eq!(ipcm, decoded_ipcm());
    }

    #[test]
    fn test_ipcm_encode() {
        let ipcm = decoded_ipcm();
        let mut buf = Vec::new();
        ipcm.encode(&mut buf).unwrap();
        assert_eq!(buf.as_slice(), ENCODED_IPCM);
    }
}
