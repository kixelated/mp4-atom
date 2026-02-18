use crate::*;

#[test]
fn av1_anim() {
    const ENCODED: &[u8] = include_bytes!("libavif_anim_q10.avif");

    let buf = &mut std::io::Cursor::new(ENCODED);
    let ftyp = Ftyp::decode(buf).expect("failed to decode ftyp");

    assert_eq!(
        ftyp,
        Ftyp {
            major_brand: b"avis".into(),
            minor_version: 0,
            compatible_brands: vec![
                b"avif".into(),
                b"avis".into(),
                b"msf1".into(),
                b"iso8".into(),
                b"mif1".into(),
                b"miaf".into(),
                b"MA1B".into()
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
                            offset: 997,
                            length: 3335
                        }]
                    }]
                }
                .into(),
                Iinf {
                    item_infos: vec![ItemInfoEntry {
                        item_id: 1,
                        item_protection_index: 0,
                        item_type: Some(b"av01".into()),
                        item_name: "Color".into(),
                        content_type: None,
                        content_encoding: None,
                        item_uri_type: None,
                        item_not_in_presentation: false
                    }]
                }
                .into(),
                Iprp {
                    ipco: Ipco {
                        properties: vec![
                            Ispe {
                                width: 768,
                                height: 512
                            }
                            .into(),
                            Pixi {
                                bits_per_channel: vec![8, 8, 8]
                            }
                            .into(),
                            Av1c {
                                seq_profile: 0,
                                seq_level_idx_0: 4,
                                seq_tier_0: false,
                                high_bitdepth: false,
                                twelve_bit: false,
                                monochrome: false,
                                chroma_subsampling_x: true,
                                chroma_subsampling_y: true,
                                chroma_sample_position: 0,
                                initial_presentation_delay: None,
                                config_obus: vec![]
                            }
                            .into(),
                            Colr::Nclx {
                                colour_primaries: 1,
                                transfer_characteristics: 13,
                                matrix_coefficients: 6,
                                full_range_flag: true
                            }
                            .into()
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
                                }
                            ]
                        }]
                    }]
                }
                .into()
            ]
        }
    );

    let moov = Moov::decode(buf).expect("failed to decode moov");
    assert_eq!(
        moov,
        Moov {
            mvhd: Mvhd {
                creation_time: 3852424385,
                modification_time: 3852424385,
                timescale: 25,
                duration: 18446744073709551615,
                rate: 1.into(),
                volume: 1.into(),
                matrix: Matrix {
                    a: 65536,
                    b: 0,
                    u: 0,
                    c: 0,
                    d: 65536,
                    v: 0,
                    x: 0,
                    y: 0,
                    w: 1073741824
                },
                next_track_id: 1
            },
            meta: None,
            mvex: None,
            trak: vec![Trak {
                tkhd: Tkhd {
                    creation_time: 3852424385,
                    modification_time: 3852424385,
                    track_id: 1,
                    duration: 18446744073709551615,
                    layer: 0,
                    alternate_group: 0,
                    enabled: true,
                    volume: 0.into(),
                    matrix: Matrix {
                        a: 65536,
                        b: 0,
                        u: 0,
                        c: 0,
                        d: 65536,
                        v: 0,
                        x: 0,
                        y: 0,
                        w: 1073741824
                    },
                    width: 768.into(),
                    height: 512.into()
                },
                edts: Some(Edts {
                    elst: Some(Elst {
                        entries: vec![ElstEntry {
                            segment_duration: 2,
                            media_time: 0,
                            media_rate: 1,
                            media_rate_fraction: 0
                        }]
                    })
                }),
                mdia: Mdia {
                    mdhd: Mdhd {
                        creation_time: 3852424385,
                        modification_time: 3852424385,
                        timescale: 25,
                        duration: 2,
                        language: "und".into()
                    },
                    hdlr: Hdlr {
                        handler: b"pict".into(),
                        name: "".into()
                    },
                    minf: Minf {
                        vmhd: Some(Vmhd {
                            graphics_mode: 0,
                            op_color: RgbColor {
                                red: 0,
                                green: 0,
                                blue: 0
                            }
                        }),
                        dinf: Dinf {
                            dref: Dref {
                                urls: vec![Url {
                                    location: "".into()
                                }]
                            }
                        },
                        stbl: Stbl {
                            stsd: Stsd {
                                codecs: vec![Av01 {
                                    visual: Visual {
                                        data_reference_index: 1,
                                        width: 768,
                                        height: 512,
                                        horizresolution: 72.into(),
                                        vertresolution: 72.into(),
                                        frame_count: 1,
                                        compressor: "AOM Coding".into(),
                                        depth: 24
                                    },
                                    av1c: Av1c {
                                        seq_profile: 0,
                                        seq_level_idx_0: 4,
                                        seq_tier_0: false,
                                        high_bitdepth: false,
                                        twelve_bit: false,
                                        monochrome: false,
                                        chroma_subsampling_x: true,
                                        chroma_subsampling_y: true,
                                        chroma_sample_position: 0,
                                        initial_presentation_delay: None,
                                        config_obus: vec![]
                                    },
                                    ccst: Some(Ccst {
                                        all_ref_pics_intra: false,
                                        intra_pred_used: true,
                                        max_ref_per_pic: 15
                                    }),
                                    colr: Some(Colr::Nclx {
                                        colour_primaries: 1,
                                        transfer_characteristics: 13,
                                        matrix_coefficients: 6,
                                        full_range_flag: true
                                    }),
                                    ..Default::default()
                                }
                                .into()],
                            },
                            stts: Stts {
                                entries: vec![SttsEntry {
                                    sample_count: 2,
                                    sample_delta: 1
                                }]
                            },
                            stss: Some(Stss { entries: vec![1] }),
                            stsc: Stsc {
                                entries: vec![StscEntry {
                                    first_chunk: 1,
                                    samples_per_chunk: 2,
                                    sample_description_index: 1
                                }]
                            },
                            stsz: Stsz {
                                samples: StszSamples::Different {
                                    sizes: vec![3335, 2803]
                                }
                            },
                            stco: Some(Stco { entries: vec![997] }),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                ..Default::default()
            }],
            ..Default::default()
        }
    );
}
