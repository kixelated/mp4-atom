mod idat;
mod iinf;
mod iloc;
mod ilst;
mod iprp;
mod iref;
mod pitm;
mod properties;

pub use idat::*;
pub use iinf::*;
pub use iloc::*;
pub use ilst::*;
pub use iprp::*;
pub use iref::*;
pub use pitm::*;
pub use properties::*;

use crate::*;

// MetaBox, ISO/IEC 14496:2022 Secion 8.11.1
// Its like a container box, but derived from FullBox

// TODO: add ItemProtectionBox, IPMPControlBox

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Meta {
    pub hdlr: Hdlr,
    pub items: Vec<Any>,
}

impl Eq for Meta {}

macro_rules! meta_atom {
    ($($atom:ident),*,) => {
        /// A trait to signify that this is a common meta atom.
        pub trait MetaAtom: AnyAtom {}

        $(impl MetaAtom for $atom {})*
    };
}

meta_atom! {
        Pitm,
        Dinf,
        Iloc,
        Iinf,
        Iprp,
        Iref,
        Idat,
        Ilst,
}

// Implement helpers to make it easier to get these atoms.
impl Meta {
    /// A helper to get a common meta atom from the items vec.
    pub fn get<T: MetaAtom>(&self) -> Option<&T> {
        self.items.iter().find_map(T::from_any_ref)
    }

    /// A helper to get a common meta atom from the items vec.
    pub fn get_mut<T: MetaAtom>(&mut self) -> Option<&mut T> {
        self.items.iter_mut().find_map(T::from_any_mut)
    }

    /// A helper to insert a common meta atom into the items vec.
    pub fn push<T: MetaAtom>(&mut self, atom: T) {
        self.items.push(atom.into_any());
    }

    /// A helper to remove a common meta atom from the items vec.
    ///
    /// This removes the first instance of the atom from the vec.
    pub fn remove<T: MetaAtom>(&mut self) -> Option<T> {
        let pos = self.items.iter().position(|a| T::from_any_ref(a).is_some());
        if let Some(pos) = pos {
            Some(T::from_any(self.items.remove(pos)).unwrap())
        } else {
            None
        }
    }
}

impl Atom for Meta {
    const KIND: FourCC = FourCC::new(b"meta");
    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        // are we a full box?
        // In Apple's QuickTime specification, the MetaBox is a regular Box.
        // In ISO 14496-12, MetaBox extends FullBox.

        if buf.remaining() < 8 {
            return Err(Error::OutOfBounds);
        }

        if buf.slice(8)[4..8] == *b"hdlr".as_ref() {
            // Apple QuickTime specification
            tracing::trace!("meta box without fullbox header");
        } else {
            // ISO 14496-12
            let _version_and_flags = u32::decode(buf)?; // version & flags
        }

        let hdlr = Hdlr::decode(buf)?;
        let mut items = Vec::new();
        while let Some(atom) = Any::decode_maybe(buf)? {
            items.push(atom);
        }

        Ok(Self { hdlr, items })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        0u32.encode(buf)?; // version & flags
        self.hdlr.encode(buf)?;
        for atom in &self.items {
            atom.encode(buf)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_empty() {
        let expected = Meta {
            hdlr: Hdlr {
                handler: b"fake".into(),
                name: "".into(),
            },
            items: Vec::new(),
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let output = Meta::decode(&mut buf).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_meta_mdir() {
        let mut expected = Meta {
            hdlr: Hdlr {
                handler: b"mdir".into(),
                name: "".into(),
            },
            items: Vec::new(),
        };

        expected.push(Pitm { item_id: 3 });
        expected.push(Dinf {
            dref: Dref {
                urls: vec![Url {
                    location: "".into(),
                }],
            },
            unexpected: vec![],
        });
        expected.push(Iloc {
            item_locations: vec![ItemLocation {
                item_id: 3,
                construction_method: 0,
                data_reference_index: 0,
                base_offset: 0,
                extents: vec![ItemLocationExtent {
                    item_reference_index: 0,
                    offset: 200,
                    length: 100,
                }],
            }],
        });
        expected.push(Iinf { item_infos: vec![] });
        expected.push(Iprp {
            ipco: Ipco { properties: vec![] },
            ipma: vec![Ipma {
                item_properties: vec![
                    PropertyAssociations {
                        item_id: 1,
                        associations: vec![
                            PropertyAssociation {
                                essential: true,
                                property_index: 1,
                            },
                            PropertyAssociation {
                                essential: false,
                                property_index: 2,
                            },
                            PropertyAssociation {
                                essential: false,
                                property_index: 3,
                            },
                            PropertyAssociation {
                                essential: false,
                                property_index: 5,
                            },
                            PropertyAssociation {
                                essential: true,
                                property_index: 4,
                            },
                        ],
                    },
                    PropertyAssociations {
                        item_id: 2,
                        associations: vec![
                            PropertyAssociation {
                                essential: true,
                                property_index: 6,
                            },
                            PropertyAssociation {
                                essential: false,
                                property_index: 3,
                            },
                            PropertyAssociation {
                                essential: false,
                                property_index: 7,
                            },
                            PropertyAssociation {
                                essential: true,
                                property_index: 8,
                            },
                            PropertyAssociation {
                                essential: true,
                                property_index: 4,
                            },
                        ],
                    },
                ],
            }],
            unexpected: vec![],
        });
        expected.push(Iref {
            references: vec![Reference {
                reference_type: b"cdsc".into(),
                from_item_id: 2,
                to_item_ids: vec![1, 3],
            }],
        });
        expected.push(Idat {
            data: vec![0x01, 0xFF, 0xFE, 0x03],
        });
        expected.push(Ilst::default());

        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let output = Meta::decode(&mut buf).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_meta_apple_quicktime() {
        // Test Apple QuickTime format meta box (without FullBox header)
        // In Apple's spec, meta box is a regular Box, not a FullBox
        // So it starts directly with hdlr instead of version+flags

        // Manually construct a meta box in Apple format
        let mut buf = Vec::new();

        // meta box header
        buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // size (placeholder, will fix)
        buf.extend_from_slice(b"meta");

        // hdlr box directly (no version/flags before it)
        let hdlr = Hdlr {
            handler: b"mdir".into(),
            name: "Apple".into(),
        };

        // Encode hdlr
        let mut hdlr_buf = Vec::new();
        hdlr.encode(&mut hdlr_buf).unwrap();
        buf.extend_from_slice(&hdlr_buf);

        // Add an ilst box
        let ilst = Ilst::default();
        let mut ilst_buf = Vec::new();
        ilst.encode(&mut ilst_buf).unwrap();
        buf.extend_from_slice(&ilst_buf);

        // Fix the meta box size
        let size = buf.len() as u32;
        buf[0..4].copy_from_slice(&size.to_be_bytes());

        // Skip the meta box header (8 bytes) to get to the body
        let mut cursor = std::io::Cursor::new(&buf[8..]);

        // Decode
        let decoded = Meta::decode_body(&mut cursor).expect("failed to decode Apple meta box");

        // Verify
        assert_eq!(decoded.hdlr.handler, FourCC::new(b"mdir"));
        assert_eq!(decoded.hdlr.name, "Apple");
        assert_eq!(decoded.items.len(), 1);
        assert!(decoded.get::<Ilst>().is_some());
    }

    #[test]
    fn test_meta_apple_with_ilst() {
        // Test a more complete Apple-style meta box with ilst containing iTunes metadata
        let mut buf = Vec::new();

        // meta box header
        buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // size (placeholder)
        buf.extend_from_slice(b"meta");

        // hdlr box (no version/flags)
        let hdlr = Hdlr {
            handler: b"mdir".into(),
            name: "".into(),
        };
        let mut hdlr_buf = Vec::new();
        hdlr.encode(&mut hdlr_buf).unwrap();
        buf.extend_from_slice(&hdlr_buf);

        // ilst box with some metadata
        let ilst = Ilst {
            name: Some(Name("Test Song".into())),
            year: Some(Year("2025".into())),
            ..Default::default()
        };

        let mut ilst_buf = Vec::new();
        ilst.encode(&mut ilst_buf).unwrap();
        buf.extend_from_slice(&ilst_buf);

        // Fix the meta box size
        let size = buf.len() as u32;
        buf[0..4].copy_from_slice(&size.to_be_bytes());

        // Decode
        let mut cursor = std::io::Cursor::new(&buf[8..]);
        let decoded =
            Meta::decode_body(&mut cursor).expect("failed to decode Apple meta with ilst");

        // Verify
        assert_eq!(decoded.hdlr.handler, FourCC::new(b"mdir"));
        let decoded_ilst = decoded.get::<Ilst>().expect("ilst not found");
        assert_eq!(decoded_ilst.name.as_ref().unwrap().0, "Test Song");
        assert_eq!(decoded_ilst.year.as_ref().unwrap().0, "2025");
    }

    #[test]
    fn test_meta_iso_vs_apple_roundtrip() {
        // Test that we can decode both ISO (with FullBox) and Apple (without) formats
        // and our encoder always produces ISO format

        let meta = Meta {
            hdlr: Hdlr {
                handler: b"mdir".into(),
                name: "Handler".into(),
            },
            items: vec![],
        };

        // Encode (produces ISO format with version/flags)
        let mut encoded = Vec::new();
        meta.encode(&mut encoded).unwrap();

        // Decode should work
        let mut cursor = std::io::Cursor::new(&encoded);
        let decoded = Meta::decode(&mut cursor).expect("failed to decode ISO format");
        assert_eq!(decoded, meta);

        // Now manually create Apple format (without version/flags)
        let mut apple_format = Vec::new();
        apple_format.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // size placeholder
        apple_format.extend_from_slice(b"meta");

        let mut hdlr_buf = Vec::new();
        meta.hdlr.encode(&mut hdlr_buf).unwrap();
        apple_format.extend_from_slice(&hdlr_buf);

        let size = apple_format.len() as u32;
        apple_format[0..4].copy_from_slice(&size.to_be_bytes());

        // Decode Apple format
        let mut cursor = std::io::Cursor::new(&apple_format);
        let decoded_apple = Meta::decode(&mut cursor).expect("failed to decode Apple format");
        assert_eq!(decoded_apple, meta);
    }
}
