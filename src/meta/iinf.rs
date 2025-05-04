use crate::*;

// ItemInformationBox. ISO/IEC 14496-12:2022 Section 8.11.6
// This is used to work out what the items are

ext! {
    name: Iinf,
    versions: [0, 1],
    flags: {}
}

ext! {
    name: ItemInfoEntry,
    versions: [0, 1, 2, 3],
    flags: {item_not_in_presentation = 0,}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemInfoEntry {
    pub item_id: u32,
    pub item_protection_index: u16,
    pub item_type: Option<FourCC>,
    pub item_name: String,
    pub content_type: Option<String>,
    pub content_encoding: Option<String>,
    pub item_not_in_presentation: bool,
}

impl AtomExt for ItemInfoEntry {
    const KIND_EXT: FourCC = FourCC::new(b"infe");

    type Ext = ItemInfoEntryExt;

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<Self::Ext> {
        // TODO: maybe work harder at versioning
        let version: ItemInfoEntryVersion = if self.item_id > u16::MAX as u32 {
            ItemInfoEntryVersion::V3
        } else {
            // version 0 or 2, since we don't support version 1 yet
            if self.item_type.is_some() {
                ItemInfoEntryVersion::V2
            } else {
                ItemInfoEntryVersion::V0
            }
        };
        if (version == ItemInfoEntryVersion::V0) || (version == ItemInfoEntryVersion::V1) {
            (self.item_id as u16).encode(buf)?;
            self.item_protection_index.encode(buf)?;
            self.item_name.as_str().encode(buf)?;
            self.content_type.clone().unwrap().as_str().encode(buf)?;
            self.content_encoding
                .clone()
                .unwrap_or("".to_string())
                .as_str()
                .encode(buf)?;
            if version == ItemInfoEntryVersion::V1 {
                unimplemented!("infe extensions are not yet supported");
            }
        } else {
            if version == ItemInfoEntryVersion::V2 {
                (self.item_id as u16).encode(buf)?;
            } else {
                self.item_id.encode(buf)?;
            }
            self.item_protection_index.encode(buf)?;
            Some(self.item_type).encode(buf)?;
            self.item_name.as_str().encode(buf)?;
            if self.item_type == Some(FourCC::new(b"mime")) {
                self.content_type.clone().unwrap().as_str().encode(buf)?;
                self.content_encoding
                    .clone()
                    .unwrap_or("".to_string())
                    .as_str()
                    .encode(buf)?;
            }
        }
        Ok(ItemInfoEntryExt {
            version,
            item_not_in_presentation: self.item_not_in_presentation,
        })
    }

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: Self::Ext) -> Result<Self> {
        let item_id: u32;
        let item_protection_index;
        let mut item_type = None;
        let item_name;
        let mut content_type = None;
        let mut content_encoding = None;
        if (ext.version == ItemInfoEntryVersion::V0) || (ext.version == ItemInfoEntryVersion::V1) {
            item_id = u16::decode(buf)? as u32;
            item_protection_index = u16::decode(buf)?;
            item_name = String::decode(buf)?;
            content_type = Some(String::decode(buf)?);
            content_encoding = Some(String::decode(buf)?);
            if ext.version == ItemInfoEntryVersion::V1 {
                unimplemented!("infe extensions are not yet supported");
            }
        } else {
            if ext.version == ItemInfoEntryVersion::V2 {
                item_id = u16::decode(buf)? as u32;
            } else {
                item_id = u32::decode(buf)?;
            }
            item_protection_index = u16::decode(buf)?;
            item_type = Some(FourCC::decode(buf)?);
            item_name = String::decode(buf)?;
            if item_type == Some(FourCC::new(b"mime")) {
                content_type = Some(String::decode(buf)?);
                content_encoding = Some(String::decode(buf)?);
            }
        }
        Ok(ItemInfoEntry {
            item_id,
            item_protection_index,
            item_type,
            item_name,
            content_type,
            content_encoding,
            item_not_in_presentation: ext.item_not_in_presentation,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Iinf {
    pub item_infos: Vec<ItemInfoEntry>,
}

impl AtomExt for Iinf {
    type Ext = IinfExt;

    const KIND_EXT: FourCC = FourCC::new(b"iinf");

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: IinfExt) -> Result<Self> {
        let mut item_infos = vec![];
        let entry_count = if ext.version == IinfVersion::V0 {
            u16::decode(buf)? as usize
        } else {
            u32::decode(buf)? as usize
        };
        for _ in 0..entry_count {
            item_infos.push(ItemInfoEntry::decode(buf)?);
        }
        Ok(Iinf { item_infos })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<IinfExt> {
        let version;
        if self.item_infos.len() > u16::MAX as usize {
            version = IinfVersion::V1;
            (self.item_infos.len() as u32).encode(buf)?
        } else {
            version = IinfVersion::V0;
            (self.item_infos.len() as u16).encode(buf)?
        }
        for item_info in &self.item_infos {
            item_info.encode(buf)?;
        }

        Ok(IinfExt { version })
    }
}
