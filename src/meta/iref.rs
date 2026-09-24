use crate::*;

// ItemReferenceBox. ISO/IEC 14496-12:2022 Section 8.11.12
// This is used to work out how the items related to each other

ext! {
    name: Iref,
    versions: [0, 1],
    flags: {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Reference {
    pub reference_type: FourCC,
    pub from_item_id: u32,
    pub to_item_ids: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Iref {
    pub references: Vec<Reference>,
}

impl AtomExt for Iref {
    type Ext = IrefExt;

    const KIND_EXT: FourCC = FourCC::new(b"iref");

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: IrefExt) -> Result<Self> {
        let mut references = vec![];
        while buf.has_remaining() {
            let box_len = u32::decode(buf)? as usize;
            let body_len = box_len.checked_sub(4).ok_or(Error::InvalidSize)?;
            if body_len > buf.remaining() {
                return Err(Error::InvalidSize);
            }

            let mut body = buf.slice(body_len);
            if ext.version == IrefVersion::V0 {
                let reference_type = FourCC::decode(&mut body)?;
                let from_item_id: u32 = u16::decode(&mut body)?.into();
                let reference_count: u16 = u16::decode(&mut body)?;
                let mut to_item_ids: Vec<u32> = vec![];
                for _ in 0..reference_count {
                    let to_item_id: u32 = u16::decode(&mut body)?.into();
                    to_item_ids.push(to_item_id);
                }
                let reference = Reference {
                    reference_type,
                    from_item_id,
                    to_item_ids,
                };
                references.push(reference);
            } else {
                let reference_type = FourCC::decode(&mut body)?;
                let from_item_id: u32 = u32::decode(&mut body)?;
                let reference_count: u16 = u16::decode(&mut body)?;
                let mut to_item_ids: Vec<u32> = vec![];
                for _ in 0..reference_count {
                    let to_item_id: u32 = u32::decode(&mut body)?;
                    to_item_ids.push(to_item_id);
                }
                let reference = Reference {
                    reference_type,
                    from_item_id,
                    to_item_ids,
                };
                references.push(reference);
            }

            if body.has_remaining() {
                return Err(Error::InvalidSize);
            }
            buf.advance(body_len);
        }
        Ok(Iref { references })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<IrefExt> {
        let mut version = IrefVersion::V0;
        'reference: for reference in &self.references {
            if reference.from_item_id > (u16::MAX as u32) {
                version = IrefVersion::V1;
                break 'reference;
            }
            for id in &reference.to_item_ids {
                if *id > (u16::MAX as u32) {
                    version = IrefVersion::V1;
                    break 'reference;
                }
            }
        }
        if version == IrefVersion::V0 {
            for reference in &self.references {
                let size = (4 + 4 + 2 + 2 + 2 * reference.to_item_ids.len()) as u32;
                size.encode(buf)?;
                reference.reference_type.encode(buf)?;
                (reference.from_item_id as u16).encode(buf)?;
                (reference.to_item_ids.len() as u16).encode(buf)?;
                for id in &reference.to_item_ids {
                    (*id as u16).encode(buf)?;
                }
            }
        } else {
            for reference in &self.references {
                let size = (4 + 4 + 4 + 2 + 4 * reference.to_item_ids.len()) as u32;
                size.encode(buf)?;
                reference.reference_type.encode(buf)?;
                reference.from_item_id.encode(buf)?;
                (reference.to_item_ids.len() as u16).encode(buf)?;
                for id in &reference.to_item_ids {
                    id.encode(buf)?;
                }
            }
        }
        Ok(IrefExt { version })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oversized_nested_reference_returns_error() {
        let body: &[u8] = &[
            0x00, 0x00, 0x00, 0x20, // nested box length exceeds the iref body
            b'd', b'i', b'm', b'g', // reference_type
            0x00, 0x01, // from_item_id
            0x00, 0x00, // reference_count
        ];

        assert!(matches!(
            Iref::decode_body_ext(
                &mut std::io::Cursor::new(body),
                IrefExt {
                    version: IrefVersion::V0,
                },
            ),
            Err(Error::InvalidSize)
        ));
    }

    #[test]
    fn nested_reference_smaller_than_header_returns_error() {
        let body: &[u8] = &[0x00, 0x00, 0x00, 0x03];

        assert!(matches!(
            Iref::decode_body_ext(
                &mut std::io::Cursor::new(body),
                IrefExt {
                    version: IrefVersion::V0,
                },
            ),
            Err(Error::InvalidSize)
        ));
    }
}
