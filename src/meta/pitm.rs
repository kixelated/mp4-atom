use crate::*;

ext! {
    name: Pitm,
    versions: [0, 1],
    flags: {}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pitm {
    pub item_id: u32,
}

impl AtomExt for Pitm {
    type Ext = PitmExt;

    const KIND_EXT: FourCC = FourCC::new(b"pitm");

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: PitmExt) -> Result<Self> {
        if ext.version == PitmVersion::V0 {
            Ok(Pitm {
                item_id: u16::decode(buf)?.into(),
            })
        } else {
            Ok(Pitm {
                item_id: u32::decode(buf)?,
            })
        }
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<PitmExt> {
        if self.item_id <= u16::MAX.into() {
            let id = self.item_id as u16;
            id.encode(buf)?;
            Ok(PitmExt {
                version: PitmVersion::V0,
            })
        } else {
            self.item_id.encode(buf)?;
            Ok(PitmExt {
                version: PitmVersion::V1,
            })
        }
    }
}
