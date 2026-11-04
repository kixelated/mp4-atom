use crate::*;

ext! {
    name: Pssh,
    versions: [0, 1],
    flags: {}
}

/// ProtectionSystemSpecificHeaderBox (`pssh`), ISO/IEC 23001-7:2023 Sect 8.1:
/// what one DRM system needs to obtain a license. It sits in `moov`, or in `moof`
/// when keys rotate.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pssh {
    pub system_id: [u8; 16],
    /// The key IDs this box applies to. `Some` encodes version 1 (even when empty),
    /// `None` version 0, so the version survives a round trip.
    pub kid: Option<Vec<[u8; 16]>>,
    /// The DRM system's own data, opaque here.
    pub data: Vec<u8>,
}

impl AtomExt for Pssh {
    type Ext = PsshExt;

    const KIND_EXT: FourCC = FourCC::new(b"pssh");

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: PsshExt) -> Result<Self> {
        let system_id = <[u8; 16]>::decode(buf)?;

        let kid = match ext.version {
            PsshVersion::V0 => None,
            PsshVersion::V1 => {
                let count = u32::decode(buf)? as usize;
                // Reject counts that cannot fit before allocating.
                if count > buf.remaining() / 16 {
                    return Err(Error::OutOfBounds);
                }
                let mut kid = Vec::with_capacity(count);
                for _ in 0..count {
                    kid.push(<[u8; 16]>::decode(buf)?);
                }
                Some(kid)
            }
        };

        let size = u32::decode(buf)? as usize;
        if size > buf.remaining() {
            return Err(Error::OutOfBounds);
        }
        let data = buf.slice(size).to_vec();
        buf.advance(size);

        Ok(Pssh {
            system_id,
            kid,
            data,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<PsshExt> {
        self.system_id.encode(buf)?;

        let version = match &self.kid {
            None => PsshVersion::V0,
            Some(kid) => {
                let count: u32 = kid
                    .len()
                    .try_into()
                    .map_err(|_| Error::TooLarge(Self::KIND))?;
                count.encode(buf)?;
                for kid in kid {
                    kid.encode(buf)?;
                }
                PsshVersion::V1
            }
        };

        let size: u32 = self
            .data
            .len()
            .try_into()
            .map_err(|_| Error::TooLarge(Self::KIND))?;
        size.encode(buf)?;
        buf.append_slice(&self.data);

        Ok(version.into())
    }
}
