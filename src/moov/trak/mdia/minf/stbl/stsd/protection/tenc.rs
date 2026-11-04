use crate::*;

ext! {
    name: Tenc,
    versions: [0, 1],
    flags: {}
}

/// TrackEncryptionBox (`tenc`), ISO/IEC 23001-7:2023 Sect 8.2.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tenc {
    /// The default encryption pattern, as `(crypt_byte_block, skip_byte_block)`.
    /// `Some` encodes version 1, which pattern schemes such as `cbcs` and `cens` use;
    /// `None` encodes version 0, so a version 1 box with a `(0, 0)` pattern stays version 1.
    /// Each value is 4 bits.
    pub default_pattern: Option<(u8, u8)>,
    pub default_is_protected: bool,
    /// 0, 8 or 16 bytes. Zero for a protected track means it uses `default_constant_iv`.
    pub default_per_sample_iv_size: u8,
    pub default_kid: [u8; 16],
    /// 8 or 16 bytes, present exactly when the track is protected with no per-sample IV,
    /// as `cbcs` does.
    pub default_constant_iv: Option<Vec<u8>>,
}

impl AtomExt for Tenc {
    type Ext = TencExt;

    const KIND_EXT: FourCC = FourCC::new(b"tenc");

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: TencExt) -> Result<Self> {
        u8::decode(buf)?; // reserved
        let pattern = u8::decode(buf)?;
        let default_pattern = match ext.version {
            TencVersion::V0 => None,
            TencVersion::V1 => Some((pattern >> 4, pattern & 0x0f)),
        };
        let default_is_protected = match u8::decode(buf)? {
            0 => false,
            1 => true,
            _ => return Err(Error::Reserved),
        };
        let default_per_sample_iv_size = u8::decode(buf)?;
        check_iv_size(default_per_sample_iv_size, PER_SAMPLE_IV_SIZES)?;
        let default_kid = <[u8; 16]>::decode(buf)?;

        let default_constant_iv = match default_is_protected && default_per_sample_iv_size == 0 {
            true => {
                let size = u8::decode(buf)?;
                check_iv_size(size, CONSTANT_IV_SIZES)?;
                let size = size as usize;
                if size > buf.remaining() {
                    return Err(Error::OutOfBounds);
                }
                let iv = buf.slice(size).to_vec();
                buf.advance(size);
                Some(iv)
            }
            false => None,
        };

        Ok(Tenc {
            default_pattern,
            default_is_protected,
            default_per_sample_iv_size,
            default_kid,
            default_constant_iv,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<TencExt> {
        let constant_iv_expected =
            self.default_is_protected && self.default_per_sample_iv_size == 0;
        if constant_iv_expected != self.default_constant_iv.is_some() {
            return Err(Error::InvalidCombination(
                "tenc carries a constant IV exactly when it is protected with no per-sample IV",
            ));
        }

        check_iv_size(self.default_per_sample_iv_size, PER_SAMPLE_IV_SIZES)?;

        0u8.encode(buf)?; // reserved
        let version = match self.default_pattern {
            None => {
                0u8.encode(buf)?; // reserved
                TencVersion::V0
            }
            Some((crypt, skip)) => {
                if crypt > 0x0f || skip > 0x0f {
                    return Err(Error::InvalidCombination(
                        "tenc pattern blocks must fit in 4 bits",
                    ));
                }
                ((crypt << 4) | skip).encode(buf)?;
                TencVersion::V1
            }
        };
        (self.default_is_protected as u8).encode(buf)?;
        self.default_per_sample_iv_size.encode(buf)?;
        self.default_kid.encode(buf)?;

        if let Some(iv) = &self.default_constant_iv {
            let size = u8::try_from(iv.len()).unwrap_or(u8::MAX);
            check_iv_size(size, CONSTANT_IV_SIZES)?;
            size.encode(buf)?;
            buf.append_slice(iv);
        }

        Ok(version.into())
    }
}

/// The IV sizes ISO/IEC 23001-7 allows, with the error for any other.
type IvSizes = (&'static [u8], &'static str);
const PER_SAMPLE_IV_SIZES: IvSizes = (
    &[0, 8, 16],
    "tenc per-sample IV size must be 0, 8 or 16 bytes",
);
const CONSTANT_IV_SIZES: IvSizes = (&[8, 16], "tenc constant IV size must be 8 or 16 bytes");

fn check_iv_size(size: u8, (allowed, error): IvSizes) -> Result<()> {
    match allowed.contains(&size) {
        true => Ok(()),
        false => Err(Error::InvalidCombination(error)),
    }
}
