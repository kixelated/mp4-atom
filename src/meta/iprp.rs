use crate::*;

// ItemPropertiesBox. ISO/IEC 14496-12:2022 Section 8.11.14
// This is used to work out what the items mean

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Iprp {
    pub ipco: Ipco,
    pub ipma: Vec<Ipma>,
}

impl Atom for Iprp {
    const KIND: FourCC = FourCC::new(b"iprp");

    nested! {
        required: [ Ipco ],
        optional: [ ],
        multiple: [ Ipma ],
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ipco {
    // Its a container, but properties (boxes) can repeat and the exact order matters
    pub properties: Vec<crate::Any>,
}

impl Atom for Ipco {
    const KIND: FourCC = FourCC::new(b"ipco");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let mut props = vec![];
        while let Some(prop) = crate::Any::decode_maybe(buf)? {
            props.push(prop);
        }
        Ok(Self { properties: props })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        for property in &self.properties {
            property.encode(buf)?
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PropertyAssociation {
    pub essential: bool,
    pub property_index: u16,
}

impl PropertyAssociation {
    fn encode<B: BufMut>(&self, buf: &mut B, prop_index_15_bit: bool) -> Result<()> {
        if prop_index_15_bit {
            let flag_and_prop_index = if self.essential {
                0x8000 | self.property_index
            } else {
                self.property_index
            };
            flag_and_prop_index.encode(buf)
        } else {
            let flag_and_prop_index = if self.essential {
                0x80 | (self.property_index as u8)
            } else {
                self.property_index as u8
            };
            flag_and_prop_index.encode(buf)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PropertyAssociations {
    pub item_id: u32,
    pub associations: Vec<PropertyAssociation>,
}

impl PropertyAssociations {
    fn encode<B: BufMut>(
        &self,
        buf: &mut B,
        version: IpmaVersion,
        prop_index_15_bit: bool,
    ) -> Result<()> {
        if version == IpmaVersion::V0 {
            (self.item_id as u16).encode(buf)?;
        } else {
            self.item_id.encode(buf)?;
        }
        let association_count: u8 = self.associations.len() as u8;
        association_count.encode(buf)?;
        for association in &self.associations {
            association.encode(buf, prop_index_15_bit)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ipma {
    pub item_properties: Vec<PropertyAssociations>,
}

ext! {
    name: Ipma,
    versions: [0, 1],
    flags: {
        prop_index_15_bits = 1,
    }
}

impl AtomExt for Ipma {
    type Ext = IpmaExt;

    const KIND_EXT: FourCC = FourCC::new(b"ipma");

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<Self::Ext> {
        let mut version = IpmaVersion::V0;
        let mut prop_index_15_bit = false;
        for item_property in &self.item_properties {
            if item_property.item_id > (u16::MAX as u32) {
                version = IpmaVersion::V1;
            }
            for association in &item_property.associations {
                if association.property_index > 0x7f {
                    prop_index_15_bit = true;
                }
            }
        }
        let entry_count: u32 = self.item_properties.len() as u32;
        entry_count.encode(buf)?;
        for item_property in &self.item_properties {
            item_property.encode(buf, version, prop_index_15_bit)?;
        }
        Ok(IpmaExt {
            version,
            prop_index_15_bits: prop_index_15_bit,
        })
    }

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: Self::Ext) -> Result<Self> {
        let entry_count = u32::decode(buf)?;
        let mut item_properties = vec![];
        for _i in 0..entry_count {
            let item_id: u32 = if ext.version == IpmaVersion::V0 {
                u16::decode(buf)? as u32
            } else {
                u32::decode(buf)?
            };
            let mut associations = vec![];
            let association_count = u8::decode(buf)?;
            // The duplicate use of i in the standard is apparently fixed in Ed 8.
            // See https://github.com/MPEGGroup/FileFormat/issues/86
            for _j in 0..association_count {
                if ext.prop_index_15_bits {
                    let flag_and_prop_index = u16::decode(buf)?;
                    let essential = (flag_and_prop_index & 0x8000) == 0x8000;
                    let property_index = flag_and_prop_index & 0x7fff;
                    associations.push(PropertyAssociation {
                        essential,
                        property_index,
                    });
                } else {
                    let flag_and_prop_index = u8::decode(buf)?;
                    let essential = (flag_and_prop_index & 0x80) == 0x80;
                    let property_index = (flag_and_prop_index & 0x7f) as u16;
                    associations.push(PropertyAssociation {
                        essential,
                        property_index,
                    });
                }
            }
            item_properties.push(PropertyAssociations {
                item_id,
                associations,
            });
        }
        Ok(Self { item_properties })
    }
}
