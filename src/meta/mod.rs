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

impl AtomExt for Meta {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"meta");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let hdlr = Hdlr::decode(buf)?;
        let mut items = Vec::new();
        while let Some(atom) = Any::decode_maybe(buf)? {
            items.push(atom);
        }

        Ok(Self { hdlr, items })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
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
}
