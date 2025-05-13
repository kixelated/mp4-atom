use crate::*;

// ItemInformationBox. ISO/IEC 14496-12:2022 Section 8.11.3
// This is used to work out where the items are

ext! {
    name: Iloc,
    versions: [0, 1, 2],
    flags: {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemLocationExtent {
    pub item_reference_index: u64,
    pub offset: u64,
    pub length: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemLocation {
    pub item_id: u32,
    pub construction_method: u8, // enum?
    pub data_reference_index: u16,
    pub base_offset: u64,
    pub extents: Vec<ItemLocationExtent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Iloc {
    pub item_locations: Vec<ItemLocation>,
}

impl AtomExt for Iloc {
    type Ext = IlocExt;

    const KIND_EXT: FourCC = FourCC::new(b"iloc");

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: IlocExt) -> Result<Self> {
        let sizes0 = u8::decode(buf)?;
        let offset_size = sizes0 >> 4;
        let length_size = sizes0 & 0x0F;
        let sizes1 = u8::decode(buf)?;
        let base_offset_size = sizes1 >> 4;
        let index_size: u8 = if ext.version == IlocVersion::V1 || ext.version == IlocVersion::V2 {
            sizes1 & 0x0F
        } else {
            0
        };

        let item_count = if ext.version == IlocVersion::V0 || ext.version == IlocVersion::V1 {
            u16::decode(buf)? as usize
        } else {
            u32::decode(buf)? as usize
        };
        let mut item_locations = vec![];
        for _i in 0..item_count {
            let item_id = if ext.version == IlocVersion::V0 || ext.version == IlocVersion::V1 {
                u16::decode(buf)? as u32
            } else {
                u32::decode(buf)?
            };
            let construction_method: u8 =
                if ext.version == IlocVersion::V1 || ext.version == IlocVersion::V2 {
                    let construction_method_packed = u16::decode(buf)?;
                    (construction_method_packed & 0x0f) as u8
                } else {
                    0
                };
            let data_reference_index = u16::decode(buf)?;
            let base_offset = match base_offset_size {
                0 => 0u64,
                4 => u32::decode(buf)? as u64,
                8 => u64::decode(buf)?,
                _ => panic!("iloc base_offset_size must be in [0,4,8]"),
            };
            let extent_count = u16::decode(buf)?;
            let mut extents = vec![];
            for _j in 0..extent_count {
                let item_reference_index: u64 =
                    if ext.version == IlocVersion::V1 || ext.version == IlocVersion::V2 {
                        match index_size {
                            0 => 0,
                            4 => u32::decode(buf)? as u64,
                            8 => u64::decode(buf)?,
                            _ => panic!("iloc index_size must be in [0,4,8]"),
                        }
                    } else {
                        0
                    };
                let extent_offset = match offset_size {
                    0 => 0u64,
                    4 => u32::decode(buf)? as u64,
                    8 => u64::decode(buf)?,
                    _ => panic!("iloc offset_size must be in [0,4,8]"),
                };
                let extent_length = match length_size {
                    0 => 0u64,
                    4 => u32::decode(buf)? as u64,
                    8 => u64::decode(buf)?,
                    _ => panic!("iloc length_size must be in [0,4,8]"),
                };
                extents.push(ItemLocationExtent {
                    item_reference_index,
                    offset: extent_offset,
                    length: extent_length,
                });
            }
            item_locations.push(ItemLocation {
                item_id,
                construction_method,
                data_reference_index,
                base_offset,
                extents,
            })
        }
        Ok(Iloc { item_locations })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<IlocExt> {
        let mut base_offset_size = 0u8;
        // TODO: work out which version and sizes we really need for this instance.
        for item_location in &self.item_locations {
            if item_location.base_offset > 0 {
                if item_location.base_offset > u32::MAX as u64 {
                    base_offset_size = 8u8;
                } else if base_offset_size != 8 {
                    base_offset_size = 4u8;
                }
            }
        }
        let version = IlocVersion::V0;
        let offset_size = 4u8;
        let length_size = 4u8;

        let index_size = 0u8;
        let size0 = (offset_size << 4) | length_size;
        let size1 = (base_offset_size << 4) | index_size;
        size0.encode(buf)?;
        size1.encode(buf)?;
        if version == IlocVersion::V0 || version == IlocVersion::V1 {
            (self.item_locations.len() as u16).encode(buf)?;
        } else {
            (self.item_locations.len() as u32).encode(buf)?;
        }
        for item_location in &self.item_locations {
            if version == IlocVersion::V0 || version == IlocVersion::V1 {
                (item_location.item_id as u16).encode(buf)?;
            } else {
                item_location.item_id.encode(buf)?;
            }
            if version == IlocVersion::V1 || version == IlocVersion::V2 {
                (item_location.construction_method as u16).encode(buf)?
            }
            item_location.data_reference_index.encode(buf)?;
            match base_offset_size {
                0 => {}
                4 => (item_location.base_offset as u32).encode(buf)?,
                8 => item_location.base_offset.encode(buf)?,
                _ => unreachable!("iloc base_offset_size must be in [0,4,8]"),
            }
            (item_location.extents.len() as u16).encode(buf)?;
            for extent in &item_location.extents {
                match index_size {
                    0 => {}
                    4 => (extent.item_reference_index as u32).encode(buf)?,
                    8 => extent.item_reference_index.encode(buf)?,
                    _ => unreachable!("iloc index_size must be in [0,4,8]"),
                }
                match offset_size {
                    0 => {}
                    4 => (extent.offset as u32).encode(buf)?,
                    8 => extent.offset.encode(buf)?,
                    _ => unreachable!("iloc offset_size must be in [0,4,8]"),
                }
                match length_size {
                    0 => {}
                    4 => (extent.length as u32).encode(buf)?,
                    8 => extent.length.encode(buf)?,
                    _ => unreachable!("iloc length_size must be in [0,4,8]"),
                }
            }
        }
        Ok(IlocExt {
            version: IlocVersion::V0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENCODED_ILOC_LIBAVIF: &[u8] = &[
        0x00, 0x00, 0x00, 0x1e, 0x69, 0x6c, 0x6f, 0x63, 0x00, 0x00, 0x00, 0x00, 0x44, 0x00, 0x00,
        0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x38, 0x00, 0x00, 0x00, 0x1a,
    ];

    #[test]
    fn test_iloc_libavif_decode() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_ILOC_LIBAVIF);

        let iloc: Iloc = Iloc {
            item_locations: vec![ItemLocation {
                item_id: 1,
                construction_method: 0,
                data_reference_index: 0,
                base_offset: 0,
                extents: vec![ItemLocationExtent {
                    item_reference_index: 0,
                    offset: 312,
                    length: 26,
                }],
            }],
        };
        let decoded = Iloc::decode(buf).unwrap();
        assert_eq!(decoded, iloc);
    }

    #[test]
    fn test_iloc_avif_encode() {
        let iloc: Iloc = Iloc {
            item_locations: vec![ItemLocation {
                item_id: 1,
                construction_method: 0,
                data_reference_index: 0,
                base_offset: 0,
                extents: vec![ItemLocationExtent {
                    item_reference_index: 0,
                    offset: 312,
                    length: 26,
                }],
            }],
        };
        let mut buf = Vec::new();
        iloc.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_ILOC_LIBAVIF);
    }
}
