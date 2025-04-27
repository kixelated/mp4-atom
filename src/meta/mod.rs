mod iinf;
mod iloc;
mod ilst;
mod iprp;
mod iref;
mod pitm;
mod properties;

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

// TODO: add DataInformationBox, ItemProtectionBox, IPMPControlBox, ItemDataBox

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Meta {
    pub hdlr: Hdlr,
    pub pitm: Option<Pitm>,
    pub iloc: Option<Iloc>,
    pub iinf: Option<Iinf>,
    pub iprp: Option<Iprp>,
    pub iref: Option<Iref>,
    pub ilst: Option<Ilst>,
    pub unknown: Vec<crate::Any>,
}

impl Eq for Meta {}

impl AtomExt for Meta {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"meta");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let hdlr = Hdlr::decode(buf)?;
        let mut pitm = None;
        let mut iloc = None;
        let mut iinf = None;
        let mut iprp = None;
        let mut iref = None;
        let mut ilst = None;
        let mut unknown_boxes = vec![];
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Pitm(atom) => pitm = Some(atom),
                Any::Iloc(atom) => iloc = Some(atom),
                Any::Iinf(atom) => iinf = Some(atom),
                Any::Iprp(atom) => iprp = Some(atom),
                Any::Iref(atom) => iref = Some(atom),
                Any::Ilst(atom) => ilst = Some(atom),
                _ => {
                    unknown_boxes.push(atom);
                }
            }
        }

        Ok(Self {
            hdlr,
            pitm,
            iloc,
            iinf,
            iprp,
            iref,
            ilst,
            unknown: unknown_boxes,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.hdlr.encode(buf)?;
        if self.pitm.is_some() {
            self.pitm.encode(buf)?;
        }
        if self.iloc.is_some() {
            self.iloc.encode(buf)?
        }
        if self.iinf.is_some() {
            self.iinf.encode(buf)?;
        }
        if self.ilst.is_some() {
            self.ilst.encode(buf)?;
        }
        if self.iprp.is_some() {
            self.iprp.encode(buf)?;
        }
        if self.iref.is_some() {
            self.iref.encode(buf)?;
        }
        for atom in &self.unknown {
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
            pitm: None,
            iloc: None,
            iinf: None,
            iprp: None,
            iref: None,
            ilst: None,
            unknown: vec![],
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let output = Meta::decode(&mut buf).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_meta_mdir() {
        let expected = Meta {
            hdlr: Hdlr {
                handler: b"mdir".into(),
                name: "".into(),
            },
            pitm: Some(Pitm { item_id: 3 }),
            iloc: Some(Iloc {
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
            }),
            iinf: Some(Iinf { item_infos: vec![] }),
            iprp: Some(Iprp {
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
            }),
            iref: Some(Iref {
                references: vec![Reference {
                    reference_type: b"cdsc".into(),
                    from_item_id: 2,
                    to_item_ids: vec![1, 3],
                }],
            }),
            ilst: Some(Ilst::default()),
            unknown: vec![],
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let output = Meta::decode(&mut buf).unwrap();
        assert_eq!(output, expected);
    }
}
