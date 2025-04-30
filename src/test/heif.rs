use crate::*;

#[test]
fn heif() {
    // Complete image from libheif
    const ENCODED: &[u8] = &[
        0x00, 0x00, 0x00, 0x1c, 0x66, 0x74, 0x79, 0x70, 0x68, 0x65, 0x69, 0x63, 0x00, 0x00, 0x00,
        0x00, 0x6d, 0x69, 0x66, 0x31, 0x68, 0x65, 0x69, 0x63, 0x6d, 0x69, 0x61, 0x66, 0x00, 0x00,
        0x02, 0x6c, 0x6d, 0x65, 0x74, 0x61, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0x68,
        0x64, 0x6c, 0x72, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x70, 0x69, 0x63, 0x74,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x0e, 0x70, 0x69, 0x74, 0x6d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x34, 0x69, 0x6c, 0x6f, 0x63, 0x00, 0x00, 0x00, 0x00, 0x44, 0x40, 0x00, 0x02, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x02, 0x90, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x2b, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x02, 0xbb, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x23, 0x00, 0x00, 0x00, 0x38, 0x69, 0x69, 0x6e, 0x66, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x15, 0x69, 0x6e, 0x66, 0x65, 0x02, 0x00, 0x00, 0x00,
        0x00, 0x01, 0x00, 0x00, 0x68, 0x76, 0x63, 0x31, 0x00, 0x00, 0x00, 0x00, 0x15, 0x69, 0x6e,
        0x66, 0x65, 0x02, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x68, 0x76, 0x63, 0x31, 0x00,
        0x00, 0x00, 0x01, 0xab, 0x69, 0x70, 0x72, 0x70, 0x00, 0x00, 0x01, 0x83, 0x69, 0x70, 0x63,
        0x6f, 0x00, 0x00, 0x00, 0x76, 0x68, 0x76, 0x63, 0x43, 0x01, 0x03, 0x70, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1e, 0xf0, 0x00, 0xfc, 0xfd, 0xf8, 0xf8, 0x00, 0x00,
        0x0f, 0x03, 0x20, 0x00, 0x01, 0x00, 0x18, 0x40, 0x01, 0x0c, 0x01, 0xff, 0xff, 0x03, 0x70,
        0x00, 0x00, 0x03, 0x00, 0x90, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x1e, 0xba, 0x02,
        0x40, 0x21, 0x00, 0x01, 0x00, 0x2a, 0x42, 0x01, 0x01, 0x03, 0x70, 0x00, 0x00, 0x03, 0x00,
        0x90, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x1e, 0xa0, 0x20, 0x81, 0x05, 0x96, 0xea,
        0xae, 0x9a, 0xe6, 0xe0, 0x21, 0xa0, 0xc0, 0x80, 0x00, 0x00, 0x03, 0x00, 0x80, 0x00, 0x00,
        0x03, 0x00, 0x84, 0x22, 0x00, 0x01, 0x00, 0x06, 0x44, 0x01, 0xc1, 0x73, 0xc1, 0x89, 0x00,
        0x00, 0x00, 0x13, 0x63, 0x6f, 0x6c, 0x72, 0x6e, 0x63, 0x6c, 0x78, 0x00, 0x01, 0x00, 0x0d,
        0x00, 0x06, 0x80, 0x00, 0x00, 0x00, 0x14, 0x69, 0x73, 0x70, 0x65, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x28, 0x63, 0x6c, 0x61,
        0x70, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
        0x00, 0x01, 0xff, 0xff, 0xff, 0xc8, 0x00, 0x00, 0x00, 0x02, 0xff, 0xff, 0xff, 0xc1, 0x00,
        0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x10, 0x70, 0x69, 0x78, 0x69, 0x00, 0x00, 0x00, 0x00,
        0x03, 0x08, 0x08, 0x08, 0x00, 0x00, 0x00, 0x71, 0x68, 0x76, 0x63, 0x43, 0x01, 0x04, 0x08,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1e, 0xf0, 0x00, 0xfc, 0xfc, 0xf8,
        0xf8, 0x00, 0x00, 0x0f, 0x03, 0x20, 0x00, 0x01, 0x00, 0x17, 0x40, 0x01, 0x0c, 0x01, 0xff,
        0xff, 0x04, 0x08, 0x00, 0x00, 0x03, 0x00, 0x9f, 0xf8, 0x00, 0x00, 0x03, 0x00, 0x00, 0x1e,
        0xba, 0x02, 0x40, 0x21, 0x00, 0x01, 0x00, 0x26, 0x42, 0x01, 0x01, 0x04, 0x08, 0x00, 0x00,
        0x03, 0x00, 0x9f, 0xf8, 0x00, 0x00, 0x03, 0x00, 0x00, 0x1e, 0xc0, 0x82, 0x04, 0x16, 0x5b,
        0xaa, 0xba, 0x6b, 0x9b, 0x02, 0x00, 0x00, 0x03, 0x00, 0x02, 0x00, 0x00, 0x03, 0x00, 0x02,
        0x10, 0x22, 0x00, 0x01, 0x00, 0x06, 0x44, 0x01, 0xc1, 0x73, 0xc1, 0x89, 0x00, 0x00, 0x00,
        0x0e, 0x70, 0x69, 0x78, 0x69, 0x00, 0x00, 0x00, 0x00, 0x01, 0x08, 0x00, 0x00, 0x00, 0x27,
        0x61, 0x75, 0x78, 0x43, 0x00, 0x00, 0x00, 0x00, 0x75, 0x72, 0x6e, 0x3a, 0x6d, 0x70, 0x65,
        0x67, 0x3a, 0x68, 0x65, 0x76, 0x63, 0x3a, 0x32, 0x30, 0x31, 0x35, 0x3a, 0x61, 0x75, 0x78,
        0x69, 0x64, 0x3a, 0x31, 0x00, 0x00, 0x00, 0x00, 0x20, 0x69, 0x70, 0x6d, 0x61, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0x05, 0x81, 0x02, 0x03, 0x05, 0x84, 0x00,
        0x02, 0x05, 0x86, 0x03, 0x07, 0x88, 0x84, 0x00, 0x00, 0x00, 0x1a, 0x69, 0x72, 0x65, 0x66,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0x61, 0x75, 0x78, 0x6c, 0x00, 0x02, 0x00,
        0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x56, 0x6d, 0x64, 0x61, 0x74, 0x00, 0x00, 0x00, 0x27,
        0x28, 0x01, 0xaf, 0x13, 0x21, 0x31, 0x96, 0xf8, 0x4e, 0x50, 0xa7, 0x96, 0xfd, 0x63, 0x02,
        0xcd, 0x0c, 0x95, 0x4c, 0x5a, 0xb9, 0x4f, 0xa5, 0x73, 0xdd, 0x0a, 0x50, 0x93, 0x72, 0x7e,
        0xb8, 0xfe, 0xf2, 0x87, 0x93, 0xc5, 0x62, 0x82, 0xe0, 0x00, 0x00, 0x00, 0x1f, 0x28, 0x01,
        0xae, 0x26, 0x42, 0x4a, 0x24, 0xe7, 0xd7, 0x0d, 0xb3, 0xfe, 0x1b, 0xc7, 0x17, 0x61, 0x55,
        0x73, 0x53, 0xb2, 0x6c, 0x20, 0x62, 0x44, 0x29, 0x12, 0x80, 0x63, 0xf5, 0xf4, 0xae,
    ];

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
            pitm: Some(Pitm { item_id: 1 }),
            dinf: None,
            iloc: Some(Iloc {
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
            }),
            iinf: Some(Iinf {
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
            }),
            iprp: Some(Iprp {
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
                                        64, 1, 12, 1, 255, 255, 3, 112, 0, 0, 3, 0, 144, 0, 0, 3,
                                        0, 0, 3, 0, 30, 186, 2, 64
                                    ]]
                                },
                                HvcCArray {
                                    completeness: false,
                                    nal_unit_type: 33,
                                    nalus: vec![vec![
                                        66, 1, 1, 3, 112, 0, 0, 3, 0, 144, 0, 0, 3, 0, 0, 3, 0, 30,
                                        160, 32, 129, 5, 150, 234, 174, 154, 230, 224, 33, 160,
                                        192, 128, 0, 0, 3, 0, 128, 0, 0, 3, 0, 132
                                    ]]
                                },
                                HvcCArray {
                                    completeness: false,
                                    nal_unit_type: 34,
                                    nalus: vec![vec![68, 1, 193, 115, 193, 137]]
                                }
                            ]
                        }),
                        any::Any::Colr(Colr::Nclx {
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
                                        64, 1, 12, 1, 255, 255, 4, 8, 0, 0, 3, 0, 159, 248, 0, 0,
                                        3, 0, 0, 30, 186, 2, 64
                                    ]]
                                },
                                HvcCArray {
                                    completeness: false,
                                    nal_unit_type: 33,
                                    nalus: vec![vec![
                                        66, 1, 1, 4, 8, 0, 0, 3, 0, 159, 248, 0, 0, 3, 0, 0, 30,
                                        192, 130, 4, 22, 91, 170, 186, 107, 155, 2, 0, 0, 3, 0, 2,
                                        0, 0, 3, 0, 2, 16
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
            }),
            iref: Some(Iref {
                references: vec![Reference {
                    reference_type: FourCC::new(b"auxl"),
                    from_item_id: 2,
                    to_item_ids: vec![1]
                }]
            }),
            ilst: None,
            unknown: vec![],
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
    const ENCODED: &[u8] = &[
        0x00, 0x00, 0x00, 0x20, 0x66, 0x74, 0x79, 0x70, 0x61, 0x76, 0x69, 0x66, 0x00, 0x00, 0x00,
        0x00, 0x61, 0x76, 0x69, 0x66, 0x6d, 0x69, 0x66, 0x31, 0x6d, 0x69, 0x61, 0x66, 0x4d, 0x41,
        0x31, 0x41, 0x00, 0x00, 0x01, 0x10, 0x6d, 0x65, 0x74, 0x61, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x21, 0x68, 0x64, 0x6c, 0x72, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x70, 0x69, 0x63, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0x70, 0x69, 0x74, 0x6d, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x1e, 0x69, 0x6c, 0x6f, 0x63, 0x00, 0x00, 0x00, 0x00, 0x44, 0x00,
        0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x38, 0x00, 0x00, 0x00,
        0x1a, 0x00, 0x00, 0x00, 0x28, 0x69, 0x69, 0x6e, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x1a, 0x69, 0x6e, 0x66, 0x65, 0x02, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
        0x00, 0x61, 0x76, 0x30, 0x31, 0x43, 0x6f, 0x6c, 0x6f, 0x72, 0x00, 0x00, 0x00, 0x00, 0x8f,
        0x69, 0x70, 0x72, 0x70, 0x00, 0x00, 0x00, 0x6d, 0x69, 0x70, 0x63, 0x6f, 0x00, 0x00, 0x00,
        0x14, 0x69, 0x73, 0x70, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0x00, 0x00,
        0x00, 0x12, 0x00, 0x00, 0x00, 0x10, 0x70, 0x69, 0x78, 0x69, 0x00, 0x00, 0x00, 0x00, 0x03,
        0x08, 0x08, 0x08, 0x00, 0x00, 0x00, 0x0c, 0x61, 0x76, 0x31, 0x43, 0x81, 0x20, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x13, 0x63, 0x6f, 0x6c, 0x72, 0x6e, 0x63, 0x6c, 0x78, 0x00, 0x01, 0x00,
        0x0d, 0x00, 0x06, 0x80, 0x00, 0x00, 0x00, 0x10, 0x70, 0x61, 0x73, 0x70, 0x00, 0x00, 0x00,
        0x02, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x09, 0x69, 0x72, 0x6f, 0x74, 0x01, 0x00,
        0x00, 0x00, 0x09, 0x69, 0x6d, 0x69, 0x72, 0x01, 0x00, 0x00, 0x00, 0x1a, 0x69, 0x70, 0x6d,
        0x61, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x07, 0x01, 0x02, 0x83,
        0x04, 0x05, 0x86, 0x87, 0x00, 0x00, 0x00, 0x22, 0x6d, 0x64, 0x61, 0x74, 0x12, 0x00, 0x0a,
        0x08, 0x38, 0x11, 0x31, 0x16, 0x10, 0x10, 0xd0, 0x69, 0x32, 0x0c, 0x16, 0x40, 0x06, 0x18,
        0x61, 0x84, 0x00, 0x01, 0x2a, 0xbe, 0xff, 0x80,
    ];

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
            pitm: Some(Pitm { item_id: 1 }),
            dinf: None,
            iloc: Some(Iloc {
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
            }),
            iinf: Some(Iinf {
                item_infos: vec![ItemInfoEntry {
                    item_id: 1,
                    item_protection_index: 0,
                    item_type: Some(FourCC::new(b"av01")),
                    item_name: "Color".into(),
                    content_type: None,
                    content_encoding: None,
                    item_not_in_presentation: false
                }]
            }),
            iprp: Some(Iprp {
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
            }),
            iref: None,
            ilst: None,
            unknown: vec![]
        }
    );

    let mdat = Mdat::decode(buf).expect("failed to decode mdat");

    let mut buf = Vec::new();
    ftyp.encode(&mut buf).expect("failed to encode ftyp");
    meta.encode(&mut buf).expect("failed to encode meta");
    mdat.encode(&mut buf).expect("failed to encode mdat");

    assert_eq!(buf, ENCODED);
}
