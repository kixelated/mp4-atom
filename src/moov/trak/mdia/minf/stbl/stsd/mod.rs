mod ac3;
mod amr;
mod audio;
mod av01;
mod btrt;
mod ccst;
mod chnl;
mod colr;
mod eac3;
mod fiel;
mod flac;
mod ftab;
mod h264;
mod hevc;
mod mp4a;
mod opus;
mod pasp;
mod pcm;
mod plaintext;
mod taic;
mod tx3g;
mod uncv;
mod visual;
mod vp9;
mod wvtt;

pub use ac3::*;
pub use amr::*;
pub use audio::*;
pub use av01::*;
pub use btrt::*;
pub use ccst::*;
pub use chnl::*;
pub use colr::*;
pub use eac3::*;
pub use fiel::*;
pub use flac::*;
pub use ftab::*;
pub use h264::*;
pub use hevc::*;
pub use mp4a::*;
pub use opus::*;
pub use pasp::*;
pub use pcm::*;
pub use plaintext::*;
pub use taic::*;
pub use tx3g::*;
pub use uncv::*;
pub use visual::*;
pub use vp9::*;
pub use wvtt::*;

use crate::*;
use derive_more::From;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Stsd {
    pub codecs: Vec<Codec>,
}

/// Called a "sample entry" in the ISOBMFF specification.
#[derive(Debug, Clone, PartialEq, Eq, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Codec {
    // H264
    Avc1(Avc1),

    // HEVC: SPS/PPS/VPS is inline
    Hev1(Hev1),

    // HEVC: SPS/PPS/VPS is in a separate atom
    Hvc1(Hvc1),

    // VP8
    Vp08(Vp08),

    // VP9
    Vp09(Vp09),

    // AV1
    Av01(Av01),

    // AAC
    Mp4a(Mp4a),

    // Text
    Tx3g(Tx3g),

    // Opus
    Opus(Opus),

    // Uncompressed video
    Uncv(Uncv),

    // FLAC audio
    Flac(Flac),

    // AC-3 audio
    Ac3(Ac3),

    // EAC-3 audio
    Eac3(Eac3),

    // Uncompressed audio
    // ipcm and fpcm are from 23003-5.
    Ipcm(Ipcm),
    Fpcm(Fpcm),
    // sowt / twos and in24 / in32 / fl32 / fl64 / lpcm are Quicktime (QTFF-2001).
    // s16l seems to be something that VLC produced at some point.
    Sowt(Sowt),
    Twos(Twos),
    Lpcm(Lpcm),
    In24(In24),
    In32(In32),
    Fl32(Fl32),
    Fl64(Fl64),
    S16l(S16l),

    // WebVTT, ISO/IEC 14496-30
    Wvtt(Wvtt),

    // 3GPP Narrowband audio (3GPP TS 26.244 or ETSI TS 126 244)
    Samr(Samr),

    // Unknown
    Unknown(FourCC),
}

impl Decode for Codec {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        let atom = Any::decode(buf)?;
        Ok(match atom {
            Any::Avc1(atom) => atom.into(),
            Any::Hev1(atom) => atom.into(),
            Any::Hvc1(atom) => atom.into(),
            Any::Vp08(atom) => atom.into(),
            Any::Vp09(atom) => atom.into(),
            Any::Mp4a(atom) => atom.into(),
            Any::Tx3g(atom) => atom.into(),
            Any::Av01(atom) => atom.into(),
            Any::Opus(atom) => atom.into(),
            Any::Uncv(atom) => atom.into(),
            Any::Flac(atom) => atom.into(),
            Any::Ac3(atom) => atom.into(),
            Any::Eac3(atom) => atom.into(),
            Any::Ipcm(atom) => atom.into(),
            Any::Fpcm(atom) => atom.into(),
            Any::Sowt(atom) => atom.into(),
            Any::Twos(atom) => atom.into(),
            Any::Lpcm(atom) => atom.into(),
            Any::In24(atom) => atom.into(),
            Any::In32(atom) => atom.into(),
            Any::Fl32(atom) => atom.into(),
            Any::Fl64(atom) => atom.into(),
            Any::S16l(atom) => atom.into(),
            Any::Wvtt(atom) => atom.into(),
            Any::Samr(atom) => atom.into(),
            unknown => {
                crate::decode_unknown(&unknown, Stsd::KIND)?;
                Self::Unknown(unknown.kind())
            }
        })
    }
}

impl Encode for Codec {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        match self {
            Self::Unknown(kind) => kind.encode(buf),
            Self::Avc1(atom) => atom.encode(buf),
            Self::Hev1(atom) => atom.encode(buf),
            Self::Hvc1(atom) => atom.encode(buf),
            Self::Vp08(atom) => atom.encode(buf),
            Self::Vp09(atom) => atom.encode(buf),
            Self::Mp4a(atom) => atom.encode(buf),
            Self::Tx3g(atom) => atom.encode(buf),
            Self::Av01(atom) => atom.encode(buf),
            Self::Opus(atom) => atom.encode(buf),
            Self::Uncv(atom) => atom.encode(buf),
            Self::Flac(atom) => atom.encode(buf),
            Self::Ac3(atom) => atom.encode(buf),
            Self::Eac3(atom) => atom.encode(buf),
            Self::Ipcm(atom) => atom.encode(buf),
            Self::Fpcm(atom) => atom.encode(buf),
            Self::Sowt(atom) => atom.encode(buf),
            Self::Twos(atom) => atom.encode(buf),
            Self::Lpcm(atom) => atom.encode(buf),
            Self::In24(atom) => atom.encode(buf),
            Self::In32(atom) => atom.encode(buf),
            Self::Fl32(atom) => atom.encode(buf),
            Self::Fl64(atom) => atom.encode(buf),
            Self::S16l(atom) => atom.encode(buf),
            Self::Wvtt(atom) => atom.encode(buf),
            Self::Samr(atom) => atom.encode(buf),
        }
    }
}

impl AtomExt for Stsd {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"stsd");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let codec_count = u32::decode(buf)?;
        let mut codecs = Vec::new();

        for _ in 0..codec_count {
            let codec = Codec::decode(buf)?;
            codecs.push(codec);
        }

        Ok(Stsd { codecs })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        (self.codecs.len() as u32).encode(buf)?;
        for codec in &self.codecs {
            codec.encode(buf)?;
        }

        Ok(())
    }
}
