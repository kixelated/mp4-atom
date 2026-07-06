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

/// The resolved sample format of a PCM sample entry.
///
/// See [`Pcm::format`] for how it is derived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PcmFormat {
    /// Whether the samples are stored big-endian.
    pub big_endian: bool,
    /// The size of each sample in bits.
    pub sample_size: u16,
    /// Whether the samples are floating point (otherwise integer).
    pub float: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pcm {
    pub fourcc: FourCC,
    pub audio: Audio,
    pub pcmc: Option<PcmC>,
    pub chnl: Option<Chnl>,
    pub btrt: Option<Btrt>,
}

impl Pcm {
    fn encode_fields<B: BufMut>(
        fourcc: FourCC,
        audio: &Audio,
        pcmc: Option<&PcmC>,
        chnl: Option<&Chnl>,
        btrt: Option<&Btrt>,
        buf: &mut B,
    ) -> Result<()> {
        // An ipcm/fpcm entry without its mandatory pcmC box is invalid (ISO/IEC 23003-5).
        if pcmc.is_none() && matches!(fourcc, Ipcm::KIND | Fpcm::KIND) {
            return Err(Error::MissingBox(PcmC::KIND));
        }

        audio.encode(buf)?;

        if let Some(pcmc) = pcmc {
            pcmc.encode(buf)?;
        }

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
                    atom => crate::decode_unknown(&atom, fourcc)?,
                }
            }

            buf.advance(size);
        }

        // ISO/IEC 23003-5 mandates the pcmC box for ipcm/fpcm. The QuickTime
        // (QTFF-2001) fourccs and s16l imply their format and don't carry one.
        if pcmc.is_none() && matches!(fourcc, Ipcm::KIND | Fpcm::KIND) {
            return Err(Error::MissingBox(PcmC::KIND));
        }

        Ok(Self {
            fourcc,
            audio,
            pcmc,
            chnl,
            btrt,
        })
    }

    pub fn encode_with_fourcc<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        Self::encode_fields(
            self.fourcc,
            &self.audio,
            self.pcmc.as_ref(),
            self.chnl.as_ref(),
            self.btrt.as_ref(),
            buf,
        )
    }

    /// Returns the resolved sample format of this entry.
    ///
    /// The `pcmC` box wins when present. Otherwise the format implied by the
    /// fourcc is used:
    /// - `twos`: big-endian integer, `sample_size` bits (8 or 16 per QTFF-2001)
    /// - `sowt`: little-endian integer, `sample_size` bits (8 or 16 per QTFF-2001)
    /// - `in24` / `in32`: big-endian integer, 24 / 32 bits
    /// - `fl32` / `fl64`: big-endian float, 32 / 64 bits
    /// - `s16l`: little-endian integer, 16 bits
    /// - `lpcm`: QTFF format flags are only defined for version 2 sound sample
    ///   descriptions, which are not decoded here; defaults to big-endian
    ///   integer, `sample_size` bits
    ///
    /// Returns `None` for an unknown fourcc, or for an `ipcm`/`fpcm` entry
    /// missing its mandatory `pcmC` box (which
    /// [`decode_with_fourcc`](Self::decode_with_fourcc) never produces).
    pub fn format(&self) -> Option<PcmFormat> {
        Self::resolve_format(self.fourcc, &self.audio, self.pcmc.as_ref())
    }

    fn resolve_format(fourcc: FourCC, audio: &Audio, pcmc: Option<&PcmC>) -> Option<PcmFormat> {
        // fl32/fl64 samples are float, as are fpcm samples (ISO/IEC 23003-5).
        let float = matches!(fourcc, Fl32::KIND | Fl64::KIND | Fpcm::KIND);

        if let Some(pcmc) = pcmc {
            return Some(PcmFormat {
                big_endian: pcmc.big_endian,
                sample_size: pcmc.sample_size as u16,
                float,
            });
        }

        let (big_endian, sample_size) = match fourcc {
            Twos::KIND | Lpcm::KIND => (true, audio.sample_size),
            Sowt::KIND => (false, audio.sample_size),
            In24::KIND => (true, 24),
            In32::KIND => (true, 32),
            Fl32::KIND => (true, 32),
            Fl64::KIND => (true, 64),
            S16l::KIND => (false, 16),
            // ipcm/fpcm require a pcmC box and unknown fourccs don't imply a format.
            _ => return None,
        };

        Some(PcmFormat {
            big_endian,
            sample_size,
            float,
        })
    }
}

macro_rules! define_pcm_sample_entry {
    ($name:ident, $fourcc:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct $name {
            pub audio: Audio,
            pub pcmc: Option<PcmC>,
            pub chnl: Option<Chnl>,
            pub btrt: Option<Btrt>,
        }

        impl $name {
            /// Returns the resolved sample format of this entry.
            ///
            /// See [`Pcm::format`] for the resolution rules.
            pub fn format(&self) -> Option<PcmFormat> {
                Pcm::resolve_format(Self::KIND, &self.audio, self.pcmc.as_ref())
            }
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
                    Self::KIND,
                    &self.audio,
                    self.pcmc.as_ref(),
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
            pcmc: Some(pcmc),
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
            pcmc: Some(pcmc),
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
            pcmc: Some(pcmc),
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
            pcmc: Some(pcmc),
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
            pcmc: Some(pcmc),
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
            pcmc: Some(pcmc),
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
            pcmc: Some(PcmC {
                big_endian: true,
                sample_size: 24,
            }),
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

    // A bare QuickTime sample entry without any child boxes, as written by
    // e.g. Sony XAVC cameras; the format is implied by the fourcc.
    const ENCODED_BARE_TWOS: &[u8] = &[
        0x00, 0x00, 0x00, 0x24, // size = 36
        0x74, 0x77, 0x6f, 0x73, // "twos"
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // reserved
        0x00, 0x01, // data_reference_index = 1
        0x00, 0x00, // version = 0
        0x00, 0x00, // revision = 0
        0x00, 0x00, 0x00, 0x00, // vendor = 0
        0x00, 0x02, // channelcount = 2
        0x00, 0x10, // samplesize = 16
        0x00, 0x00, // compression_id = 0
        0x00, 0x00, // packet_size = 0
        0xbb, 0x80, 0x00, 0x00, // samplerate = 48000 << 16
    ];

    #[test]
    fn test_bare_twos_decode() {
        let buf = &mut std::io::Cursor::new(ENCODED_BARE_TWOS);
        let twos = Twos::decode(buf).expect("failed to decode twos");
        assert_eq!(
            twos,
            Twos {
                audio: Audio {
                    data_reference_index: 1,
                    channel_count: 2,
                    sample_size: 16,
                    sample_rate: FixedPoint::new(48000, 0),
                },
                pcmc: None,
                chnl: None,
                btrt: None,
            }
        );
        assert_eq!(
            twos.format(),
            Some(PcmFormat {
                big_endian: true,
                sample_size: 16,
                float: false,
            })
        );
    }

    #[test]
    fn test_bare_twos_roundtrip() {
        let buf = &mut std::io::Cursor::new(ENCODED_BARE_TWOS);
        let twos = Twos::decode(buf).expect("failed to decode twos");

        let mut encoded = Vec::new();
        twos.encode(&mut encoded).unwrap();
        assert_eq!(encoded.as_slice(), ENCODED_BARE_TWOS);
    }

    #[test]
    fn test_ipcm_missing_pcmc_decode() {
        // The same bare entry, but ipcm requires a pcmC box (ISO/IEC 23003-5).
        let mut encoded = ENCODED_BARE_TWOS.to_vec();
        encoded[4..8].copy_from_slice(b"ipcm");

        let buf = &mut std::io::Cursor::new(&encoded);
        assert!(matches!(
            Ipcm::decode(buf),
            Err(Error::MissingBox(PcmC::KIND))
        ));
    }

    #[test]
    fn test_ipcm_missing_pcmc_encode() {
        let ipcm = Ipcm {
            audio: Audio {
                data_reference_index: 1,
                channel_count: 2,
                sample_size: 16,
                sample_rate: 48000.into(),
            },
            pcmc: None,
            chnl: None,
            btrt: None,
        };

        let mut buf = Vec::new();
        assert!(matches!(
            ipcm.encode(&mut buf),
            Err(Error::MissingBox(PcmC::KIND))
        ));
    }
}
