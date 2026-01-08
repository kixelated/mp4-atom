use crate::*;

#[test]
fn heif() {
    // Complete image from libheif
    const ENCODED: &[u8] = include_bytes!("image.heif");

    let buf = &mut std::io::Cursor::new(ENCODED);
    let ftyp = Ftyp::decode(buf).expect("failed to decode ftyp");

    assert_eq!(
        ftyp,
        Ftyp {
            major_brand: b"heic".into(),
            minor_version: 0,
            compatible_brands: vec![b"mif1".into(), b"heic".into(), b"miaf".into()],
        }
    );

    let meta = Meta::decode(buf).expect("failed to decode meta");
    assert_eq!(
        meta,
        Meta {
            hdlr: Hdlr {
                handler: b"pict".into(),
                name: "".into()
            },
            items: vec![
                Pitm { item_id: 1 }.into(),
                Iloc {
                    item_locations: vec![
                        ItemLocation {
                            item_id: 1,
                            construction_method: 0,
                            data_reference_index: 0,
                            base_offset: 656,
                            extents: vec![ItemLocationExtent {
                                item_reference_index: 0,
                                offset: 0,
                                length: 43
                            }]
                        },
                        ItemLocation {
                            item_id: 2,
                            construction_method: 0,
                            data_reference_index: 0,
                            base_offset: 699,
                            extents: vec![ItemLocationExtent {
                                item_reference_index: 0,
                                offset: 0,
                                length: 35
                            }]
                        }
                    ]
                }
                .into(),
                Iinf {
                    item_infos: [
                        ItemInfoEntry {
                            item_id: 1,
                            item_protection_index: 0,
                            item_type: Some(FourCC::new(b"hvc1")),
                            item_name: "".into(),
                            content_type: None,
                            content_encoding: None,
                            item_not_in_presentation: false,
                        },
                        ItemInfoEntry {
                            item_id: 2,
                            item_protection_index: 0,
                            item_type: Some(FourCC::new(b"hvc1")),
                            item_name: "".into(),
                            content_type: None,
                            content_encoding: None,
                            item_not_in_presentation: false,
                        },
                    ]
                    .to_vec()
                }
                .into(),
                Iprp {
                    ipco: Ipco {
                        properties: vec![
                            any::Any::Hvcc(Hvcc {
                                configuration_version: 1,
                                general_profile_space: 0,
                                general_tier_flag: false,
                                general_profile_idc: 3,
                                general_profile_compatibility_flags: [112, 0, 0, 0],
                                general_constraint_indicator_flags: [0, 0, 0, 0, 0, 0],
                                general_level_idc: 30,
                                min_spatial_segmentation_idc: 0,
                                parallelism_type: 0,
                                chroma_format_idc: 1,
                                bit_depth_luma_minus8: 0,
                                bit_depth_chroma_minus8: 0,
                                avg_frame_rate: 0,
                                constant_frame_rate: 0,
                                num_temporal_layers: 1,
                                temporal_id_nested: true,
                                length_size_minus_one: 3,
                                arrays: vec![
                                    HvcCArray {
                                        completeness: false,
                                        nal_unit_type: 32,
                                        nalus: vec![vec![
                                            64, 1, 12, 1, 255, 255, 3, 112, 0, 0, 3, 0, 144, 0, 0,
                                            3, 0, 0, 3, 0, 30, 186, 2, 64
                                        ]]
                                    },
                                    HvcCArray {
                                        completeness: false,
                                        nal_unit_type: 33,
                                        nalus: vec![vec![
                                            66, 1, 1, 3, 112, 0, 0, 3, 0, 144, 0, 0, 3, 0, 0, 3, 0,
                                            30, 160, 32, 129, 5, 150, 234, 174, 154, 230, 224, 33,
                                            160, 192, 128, 0, 0, 3, 0, 128, 0, 0, 3, 0, 132
                                        ]]
                                    },
                                    HvcCArray {
                                        completeness: false,
                                        nal_unit_type: 34,
                                        nalus: vec![vec![68, 1, 193, 115, 193, 137]]
                                    }
                                ]
                            }),
                            Any::Colr(Colr::Nclx {
                                colour_primaries: 1,
                                transfer_characteristics: 13,
                                matrix_coefficients: 6,
                                full_range_flag: true
                            }),
                            Any::Ispe(Ispe {
                                width: 64,
                                height: 64
                            }),
                            Any::Clap(Clap {
                                clean_aperture_width_n: 8,
                                clean_aperture_width_d: 1,
                                clean_aperture_height_n: 1,
                                clean_aperture_height_d: 1,
                                horiz_off_n: -56,
                                horiz_off_d: 2,
                                vert_off_n: -63,
                                vert_off_d: 2
                            }),
                            Any::Pixi(Pixi {
                                bits_per_channel: vec![8, 8, 8]
                            }),
                            Any::Hvcc(Hvcc {
                                configuration_version: 1,
                                general_profile_space: 0,
                                general_tier_flag: false,
                                general_profile_idc: 4,
                                general_profile_compatibility_flags: [8, 0, 0, 0],
                                general_constraint_indicator_flags: [0, 0, 0, 0, 0, 0],
                                general_level_idc: 30,
                                min_spatial_segmentation_idc: 0,
                                parallelism_type: 0,
                                chroma_format_idc: 0,
                                bit_depth_luma_minus8: 0,
                                bit_depth_chroma_minus8: 0,
                                avg_frame_rate: 0,
                                constant_frame_rate: 0,
                                num_temporal_layers: 1,
                                temporal_id_nested: true,
                                length_size_minus_one: 3,
                                arrays: vec![
                                    HvcCArray {
                                        completeness: false,
                                        nal_unit_type: 32,
                                        nalus: vec![vec![
                                            64, 1, 12, 1, 255, 255, 4, 8, 0, 0, 3, 0, 159, 248, 0,
                                            0, 3, 0, 0, 30, 186, 2, 64
                                        ]]
                                    },
                                    HvcCArray {
                                        completeness: false,
                                        nal_unit_type: 33,
                                        nalus: vec![vec![
                                            66, 1, 1, 4, 8, 0, 0, 3, 0, 159, 248, 0, 0, 3, 0, 0,
                                            30, 192, 130, 4, 22, 91, 170, 186, 107, 155, 2, 0, 0,
                                            3, 0, 2, 0, 0, 3, 0, 2, 16
                                        ]]
                                    },
                                    HvcCArray {
                                        completeness: false,
                                        nal_unit_type: 34,
                                        nalus: vec![vec![68, 1, 193, 115, 193, 137]]
                                    }
                                ]
                            }),
                            Any::Pixi(Pixi {
                                bits_per_channel: vec![8]
                            }),
                            Any::Auxc(Auxc {
                                aux_type: "urn:mpeg:hevc:2015:auxid:1".into(),
                                aux_subtype: vec![]
                            }),
                        ]
                    },
                    ipma: vec![Ipma {
                        item_properties: vec![
                            PropertyAssociations {
                                item_id: 1,
                                associations: vec![
                                    PropertyAssociation {
                                        essential: true,
                                        property_index: 1
                                    },
                                    PropertyAssociation {
                                        essential: false,
                                        property_index: 2
                                    },
                                    PropertyAssociation {
                                        essential: false,
                                        property_index: 3
                                    },
                                    PropertyAssociation {
                                        essential: false,
                                        property_index: 5
                                    },
                                    PropertyAssociation {
                                        essential: true,
                                        property_index: 4
                                    }
                                ]
                            },
                            PropertyAssociations {
                                item_id: 2,
                                associations: vec![
                                    PropertyAssociation {
                                        essential: true,
                                        property_index: 6
                                    },
                                    PropertyAssociation {
                                        essential: false,
                                        property_index: 3
                                    },
                                    PropertyAssociation {
                                        essential: false,
                                        property_index: 7
                                    },
                                    PropertyAssociation {
                                        essential: true,
                                        property_index: 8
                                    },
                                    PropertyAssociation {
                                        essential: true,
                                        property_index: 4
                                    }
                                ]
                            }
                        ]
                    }]
                }
                .into(),
                Iref {
                    references: vec![Reference {
                        reference_type: FourCC::new(b"auxl"),
                        from_item_id: 2,
                        to_item_ids: vec![1]
                    }]
                }
                .into(),
            ],
        }
    );

    let mdat = Mdat::decode(buf).expect("failed to decode mdat");

    let mut buf = Vec::new();
    ftyp.encode(&mut buf).expect("failed to encode ftyp");
    meta.encode(&mut buf).expect("failed to encode meta");
    mdat.encode(&mut buf).expect("failed to encode mdat");

    assert_eq!(buf, ENCODED);
}

#[test]
fn avif() {
    // Complete image from libavif
    const ENCODED: &[u8] = include_bytes!("image.avif");

    let buf = &mut std::io::Cursor::new(ENCODED);
    let ftyp = Ftyp::decode(buf).expect("failed to decode ftyp");

    assert_eq!(
        ftyp,
        Ftyp {
            major_brand: b"avif".into(),
            minor_version: 0,
            compatible_brands: vec![
                b"avif".into(),
                b"mif1".into(),
                b"miaf".into(),
                b"MA1A".into()
            ],
        }
    );

    let meta = Meta::decode(buf).expect("failed to decode meta");
    assert_eq!(
        meta,
        Meta {
            hdlr: Hdlr {
                handler: b"pict".into(),
                name: "".into()
            },
            items: vec![
                Pitm { item_id: 1 }.into(),
                Iloc {
                    item_locations: vec![ItemLocation {
                        item_id: 1,
                        construction_method: 0,
                        data_reference_index: 0,
                        base_offset: 0,
                        extents: vec![ItemLocationExtent {
                            item_reference_index: 0,
                            offset: 312,
                            length: 26
                        }]
                    }]
                }
                .into(),
                Iinf {
                    item_infos: vec![ItemInfoEntry {
                        item_id: 1,
                        item_protection_index: 0,
                        item_type: Some(FourCC::new(b"av01")),
                        item_name: "Color".into(),
                        content_type: None,
                        content_encoding: None,
                        item_not_in_presentation: false
                    }]
                }
                .into(),
                Iprp {
                    ipco: Ipco {
                        properties: vec![
                            Any::Ispe(Ispe {
                                width: 25,
                                height: 18
                            }),
                            Any::Pixi(Pixi {
                                bits_per_channel: vec![8, 8, 8]
                            }),
                            Any::Av1c(Av1c {
                                seq_profile: 1,
                                seq_level_idx_0: 0,
                                seq_tier_0: false,
                                high_bitdepth: false,
                                twelve_bit: false,
                                monochrome: false,
                                chroma_subsampling_x: false,
                                chroma_subsampling_y: false,
                                chroma_sample_position: 0,
                                initial_presentation_delay: None,
                                config_obus: vec![]
                            }),
                            Any::Colr(Colr::Nclx {
                                colour_primaries: 1,
                                transfer_characteristics: 13,
                                matrix_coefficients: 6,
                                full_range_flag: true
                            }),
                            Any::Pasp(Pasp {
                                h_spacing: 2,
                                v_spacing: 3
                            }),
                            Any::Irot(Irot { angle: 1 }),
                            Any::Imir(Imir { axis: 1 }),
                        ]
                    },
                    ipma: vec![Ipma {
                        item_properties: vec![PropertyAssociations {
                            item_id: 1,
                            associations: vec![
                                PropertyAssociation {
                                    essential: false,
                                    property_index: 1
                                },
                                PropertyAssociation {
                                    essential: false,
                                    property_index: 2
                                },
                                PropertyAssociation {
                                    essential: true,
                                    property_index: 3
                                },
                                PropertyAssociation {
                                    essential: false,
                                    property_index: 4
                                },
                                PropertyAssociation {
                                    essential: false,
                                    property_index: 5
                                },
                                PropertyAssociation {
                                    essential: true,
                                    property_index: 6
                                },
                                PropertyAssociation {
                                    essential: true,
                                    property_index: 7
                                }
                            ]
                        }]
                    }]
                }
                .into(),
            ],
        }
    );

    let mdat = Mdat::decode(buf).expect("failed to decode mdat");

    let mut buf = Vec::new();
    ftyp.encode(&mut buf).expect("failed to encode ftyp");
    meta.encode(&mut buf).expect("failed to encode meta");
    mdat.encode(&mut buf).expect("failed to encode mdat");

    assert_eq!(buf, ENCODED);
}
