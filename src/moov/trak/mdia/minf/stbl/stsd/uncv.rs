use crate::*;

use super::{Btrt, Pasp, Visual};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Uncv {
    pub visual: Visual,
    pub cmpd: Option<Cmpd>,
    pub uncc: UncC,
    pub btrt: Option<Btrt>,
    pub ccst: Option<Ccst>,
    pub pasp: Option<Pasp>,
    #[cfg(feature = "fault-tolerant")]
    pub unexpected: Vec<Any>,
}

impl Atom for Uncv {
    const KIND: FourCC = FourCC::new(b"uncv");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let visual = Visual::decode(buf)?;

        let mut ccst = None;
        let mut cmpd = None;
        let mut uncc = None;
        let mut btrt = None;
        let mut pasp = None;
        #[cfg(feature = "fault-tolerant")]
        let mut unexpected = Vec::new();

        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Cmpd(atom) => cmpd = atom.into(),
                Any::UncC(atom) => uncc = atom.into(),
                Any::Btrt(atom) => btrt = atom.into(),
                Any::Ccst(atom) => ccst = atom.into(),
                Any::Pasp(atom) => pasp = atom.into(),
                _ => {
                    tracing::warn!("unknown atom: {:?}", atom);
                    #[cfg(feature = "fault-tolerant")]
                    unexpected.push(atom);

                    #[cfg(not(feature = "fault-tolerant"))]
                    return Err(Error::UnexpectedBox(atom.kind()));
                }
            }
        }

        Ok(Uncv {
            visual,
            cmpd,
            uncc: uncc.ok_or(Error::MissingBox(UncC::KIND))?,
            btrt,
            ccst,
            pasp,
            #[cfg(feature = "fault-tolerant")]
            unexpected,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.visual.encode(buf)?;
        if self.cmpd.is_some() {
            self.cmpd.encode(buf)?;
        }
        self.uncc.encode(buf)?;
        if self.btrt.is_some() {
            self.btrt.encode(buf)?;
        }
        if self.ccst.is_some() {
            self.ccst.encode(buf)?;
        }
        if self.pasp.is_some() {
            self.pasp.encode(buf)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Component {
    pub component_type: u16,
    pub component_type_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cmpd {
    pub components: Vec<Component>,
}

impl Atom for Cmpd {
    const KIND: FourCC = FourCC::new(b"cmpd");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let component_count = u32::decode(buf)?;
        let mut components: Vec<Component> = Vec::with_capacity(component_count as usize);
        for _ in 0..component_count {
            let component_type = u16::decode(buf)?;
            if component_type >= 0x8000 {
                let component_type_uri = String::decode(buf)?;
                components.push(Component {
                    component_type,
                    component_type_uri: Some(component_type_uri),
                });
            } else {
                components.push(Component {
                    component_type,
                    component_type_uri: None,
                });
            }
        }
        Ok(Cmpd { components })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let component_count: u32 = self.components.len() as u32;
        component_count.encode(buf)?;
        for component in &self.components {
            component.component_type.encode(buf)?;
            if component.component_type >= 0x8000 {
                let component_type_uri = component
                    .component_type_uri
                    .as_ref()
                    .expect("Expected valid URI when component_type is >= 0x8000");
                component_type_uri.as_str().encode(buf)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UncompressedComponent {
    pub component_index: u16,
    pub component_bit_depth_minus_one: u8,
    pub component_format: u8,
    pub component_align_size: u8,
}

ext! {
    name: UncC,
    versions: [0, 1],
    flags: {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UncC {
    V1 {
        profile: FourCC,
    },
    V0 {
        profile: FourCC,
        components: Vec<UncompressedComponent>,
        sampling_type: u8,   // TODO: enum?
        interleave_type: u8, // TODO: enum?
        block_size: u8,
        components_little_endian: bool,
        block_pad_lsb: bool,
        block_little_endian: bool,
        block_reversed: bool,
        pad_unknown: bool,
        pixel_size: u32,
        row_align_size: u32,
        tile_align_size: u32,
        num_tile_cols_minus_one: u32,
        num_tile_rows_minus_one: u32,
    },
}

impl AtomExt for UncC {
    const KIND_EXT: FourCC = FourCC::new(b"uncC");

    type Ext = UncCExt;

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: UncCExt) -> Result<Self> {
        match ext.version {
            UncCVersion::V1 => Ok(UncC::V1 {
                profile: FourCC::decode(buf)?,
            }),
            UncCVersion::V0 => {
                let profile = FourCC::decode(buf)?;
                let component_count = u32::decode(buf)?;
                let mut components = Vec::with_capacity(component_count as usize);
                for _ in 0..component_count {
                    components.push(UncompressedComponent {
                        component_index: u16::decode(buf)?,
                        component_bit_depth_minus_one: u8::decode(buf)?,
                        component_format: u8::decode(buf)?,
                        component_align_size: u8::decode(buf)?,
                    });
                }
                let sampling_type = u8::decode(buf)?;
                let interleave_type = u8::decode(buf)?;
                let block_size = u8::decode(buf)?;
                let flag_bits = u8::decode(buf)?;
                let components_little_endian = flag_bits & 0x80 == 0x80;
                let block_pad_lsb = flag_bits & 0x40 == 0x40;
                let block_little_endian = flag_bits & 0x20 == 0x20;
                let block_reversed = flag_bits & 0x10 == 0x10;
                let pad_unknown = flag_bits & 0x08 == 0x08;
                let pixel_size = u32::decode(buf)?;
                let row_align_size = u32::decode(buf)?;
                let tile_align_size = u32::decode(buf)?;
                let num_tile_cols_minus_one = u32::decode(buf)?;
                let num_tile_rows_minus_one = u32::decode(buf)?;
                Ok(UncC::V0 {
                    profile,
                    components,
                    sampling_type,
                    interleave_type,
                    block_size,
                    components_little_endian,
                    block_pad_lsb,
                    block_little_endian,
                    block_reversed,
                    pad_unknown,
                    pixel_size,
                    row_align_size,
                    tile_align_size,
                    num_tile_cols_minus_one,
                    num_tile_rows_minus_one,
                })
            }
        }
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<UncCExt> {
        match self {
            UncC::V1 { profile } => {
                profile.encode(buf)?;
                Ok(UncCExt {
                    version: UncCVersion::V1,
                })
            }
            UncC::V0 {
                profile,
                components,
                sampling_type,
                interleave_type,
                block_size,
                components_little_endian,
                block_pad_lsb,
                block_little_endian,
                block_reversed,
                pad_unknown,
                pixel_size,
                row_align_size,
                tile_align_size,
                num_tile_cols_minus_one,
                num_tile_rows_minus_one,
            } => {
                profile.encode(buf)?;
                let component_count: u32 = components.len() as u32;
                component_count.encode(buf)?;
                for component in components {
                    component.component_index.encode(buf)?;
                    component.component_bit_depth_minus_one.encode(buf)?;
                    component.component_format.encode(buf)?;
                    component.component_align_size.encode(buf)?;
                }
                sampling_type.encode(buf)?;
                interleave_type.encode(buf)?;
                block_size.encode(buf)?;
                let mut flags: u8 = 0x00;
                if *components_little_endian {
                    flags |= 0x80u8;
                }
                if *block_pad_lsb {
                    flags |= 0x40u8;
                }
                if *block_little_endian {
                    flags |= 0x20u8;
                }
                if *block_reversed {
                    flags |= 0x10u8;
                }
                if *pad_unknown {
                    flags |= 0x08u8;
                }
                flags.encode(buf)?;
                pixel_size.encode(buf)?;
                row_align_size.encode(buf)?;
                tile_align_size.encode(buf)?;
                num_tile_cols_minus_one.encode(buf)?;
                num_tile_rows_minus_one.encode(buf)?;
                Ok(UncCExt {
                    version: UncCVersion::V0,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENCODED_CMPD: &[u8] = &[
        0x00, 0x00, 0x00, 0x12, 0x63, 0x6d, 0x70, 0x64, 0x00, 0x00, 0x00, 0x03, 0x00, 0x04, 0x00,
        0x05, 0x00, 0x06,
    ];

    const ENCODED_UNCC: &[u8] = &[
        0x00, 0x00, 0x00, 0x3b, 0x75, 0x6e, 0x63, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x01, 0x07, 0x00, 0x00,
        0x00, 0x02, 0x07, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn test_cmpd_decode() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_CMPD);

        let cmpd = Cmpd::decode(buf).expect("failed to decode cmpd");

        assert_eq!(
            cmpd,
            Cmpd {
                components: vec![
                    Component {
                        component_type: 4,
                        component_type_uri: None
                    },
                    Component {
                        component_type: 5,
                        component_type_uri: None
                    },
                    Component {
                        component_type: 6,
                        component_type_uri: None
                    }
                ]
            },
        );
    }

    #[test]
    fn test_cmpd_encode() {
        let cmpd = Cmpd {
            components: vec![
                Component {
                    component_type: 4,
                    component_type_uri: None,
                },
                Component {
                    component_type: 5,
                    component_type_uri: None,
                },
                Component {
                    component_type: 6,
                    component_type_uri: None,
                },
            ],
        };

        let mut buf = Vec::new();
        cmpd.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_CMPD);
    }

    #[test]
    fn test_uncc_decode() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_UNCC);

        let uncc = UncC::decode(buf).expect("failed to decode uncC");

        assert_eq!(
            uncc,
            UncC::V0 {
                profile: FourCC::new(b"\0\0\0\0"),
                components: vec![
                    UncompressedComponent {
                        component_index: 0,
                        component_bit_depth_minus_one: 7,
                        component_format: 0,
                        component_align_size: 0
                    },
                    UncompressedComponent {
                        component_index: 1,
                        component_bit_depth_minus_one: 7,
                        component_format: 0,
                        component_align_size: 0
                    },
                    UncompressedComponent {
                        component_index: 2,
                        component_bit_depth_minus_one: 7,
                        component_format: 0,
                        component_align_size: 0
                    },
                ],
                sampling_type: 0,
                interleave_type: 1,
                block_size: 0,
                components_little_endian: false,
                block_pad_lsb: false,
                block_little_endian: false,
                block_reversed: false,
                pad_unknown: false,
                pixel_size: 0,
                row_align_size: 0,
                tile_align_size: 0,
                num_tile_cols_minus_one: 0,
                num_tile_rows_minus_one: 0
            }
        );
    }

    #[test]
    fn test_uncc_encode() {
        let uncc = UncC::V0 {
            profile: FourCC::new(b"\0\0\0\0"),
            components: vec![
                UncompressedComponent {
                    component_index: 0,
                    component_bit_depth_minus_one: 7,
                    component_format: 0,
                    component_align_size: 0,
                },
                UncompressedComponent {
                    component_index: 1,
                    component_bit_depth_minus_one: 7,
                    component_format: 0,
                    component_align_size: 0,
                },
                UncompressedComponent {
                    component_index: 2,
                    component_bit_depth_minus_one: 7,
                    component_format: 0,
                    component_align_size: 0,
                },
            ],
            sampling_type: 0,
            interleave_type: 1,
            block_size: 0,
            components_little_endian: false,
            block_pad_lsb: false,
            block_little_endian: false,
            block_reversed: false,
            pad_unknown: false,
            pixel_size: 0,
            row_align_size: 0,
            tile_align_size: 0,
            num_tile_cols_minus_one: 0,
            num_tile_rows_minus_one: 0,
        };

        let mut buf = Vec::new();
        uncc.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_UNCC);
    }
}
