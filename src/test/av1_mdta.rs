use crate::*;

#[test]
fn av1_mdta() {
    // Created from av1.mp4 with the following command:
    // ffmpeg -i av1.mp4 -c copy -movflags use_metadata_tags \
    //     -metadata com.example.test="some value" av1_mdta.mp4
    const ENCODED: &[u8] = include_bytes!("av1_mdta.mp4");

    let buf = &mut std::io::Cursor::new(ENCODED);
    let ftyp = Ftyp::decode(buf).expect("failed to decode ftyp");

    assert_eq!(
        ftyp,
        Ftyp {
            major_brand: b"isom".into(),
            minor_version: 512,
            compatible_brands: vec![
                b"isom".into(),
                b"av01".into(),
                b"iso2".into(),
                b"mp41".into()
            ],
        }
    );

    let free = Free::decode(buf).expect("failed to decode free");

    let mdat = Mdat::decode(buf).expect("failed to decode mdat");

    let moov = Moov::decode(buf).expect("failed to decode moov");
    assert_eq!(
        moov,
        Moov {
            mvhd: Mvhd {
                creation_time: 0,
                modification_time: 0,
                timescale: 25000,
                duration: 1000,
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
                next_track_id: 2
            },
            mvex: None,
            trak: vec![Trak {
                tkhd: Tkhd {
                    creation_time: 0,
                    modification_time: 0,
                    track_id: 1,
                    duration: 1000,
                    layer: 0,
                    alternate_group: 0,
                    enabled: true,
                    in_movie: true,
                    size_is_aspect_ratio: false,
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
                    width: 1920.into(),
                    height: 1080.into()
                },
                edts: Some(Edts {
                    elst: Some(Elst {
                        entries: vec![ElstEntry {
                            segment_duration: 1000,
                            media_time: Some(0),
                            media_rate: 1.into()
                        }]
                    })
                }),
                mdia: Mdia {
                    mdhd: Mdhd {
                        creation_time: 0,
                        modification_time: 0,
                        timescale: 25000,
                        duration: 1000,
                        language: "und".into()
                    },
                    hdlr: Hdlr {
                        handler: b"vide".into(),
                        name: "obu@GPAC2.1-DEV-rev199-g8e29f6e8b-github_master".into()
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
                                        width: 1920,
                                        height: 1080,
                                        horizresolution: 72.into(),
                                        vertresolution: 72.into(),
                                        frame_count: 1,
                                        compressor: "".into(),
                                        depth: 24
                                    },
                                    av1c: Av1c {
                                        seq_profile: 0,
                                        seq_level_idx_0: 9,
                                        seq_tier_0: false,
                                        high_bitdepth: true,
                                        twelve_bit: false,
                                        monochrome: false,
                                        chroma_subsampling_x: true,
                                        chroma_subsampling_y: true,
                                        chroma_sample_position: 0,
                                        initial_presentation_delay: None,
                                        config_obus: vec![
                                            10, 11, 0, 0, 0, 74, 171, 191, 195, 119, 255, 231, 1
                                        ]
                                    },
                                    btrt: Some(Btrt {
                                        buffer_size_db: 0,
                                        max_bitrate: 70500,
                                        avg_bitrate: 50400
                                    }),
                                    pasp: Some(Pasp {
                                        h_spacing: 1,
                                        v_spacing: 1
                                    }),
                                    ..Default::default()
                                }
                                .into()],
                            },
                            stts: Stts {
                                entries: vec![SttsEntry {
                                    sample_count: 1,
                                    sample_delta: 1000
                                }]
                            },
                            ctts: None,
                            stss: None,
                            stsc: Stsc {
                                entries: vec![StscEntry {
                                    first_chunk: 1,
                                    samples_per_chunk: 1,
                                    sample_description_index: 1
                                }]
                            },
                            stsz: Stsz {
                                samples: StszSamples::Identical {
                                    count: 1,
                                    size: 252
                                }
                            },
                            stco: Some(Stco { entries: vec![48] }),
                            co64: None,
                            sbgp: vec![],
                            sgpd: vec![],
                            subs: vec![],
                            saio: vec![],
                            saiz: vec![],
                            cslg: None,
                        },
                        ..Default::default()
                    }
                },
                ..Default::default()
            }],
            udta: Some(Udta {
                meta: Some(Meta {
                    hdlr: Hdlr {
                        handler: FourCC::new(b"mdta"),
                        name: "".into()
                    },
                    items: vec![
                        Keys {
                            entries: vec![
                                KeyEntry {
                                    key_namespace: FourCC::new(b"mdta"),
                                    key_value: "major_brand".into()
                                },
                                KeyEntry {
                                    key_namespace: FourCC::new(b"mdta"),
                                    key_value: "minor_version".into()
                                },
                                KeyEntry {
                                    key_namespace: FourCC::new(b"mdta"),
                                    key_value: "compatible_brands".into()
                                },
                                KeyEntry {
                                    key_namespace: FourCC::new(b"mdta"),
                                    key_value: "com.example.test".into()
                                },
                                KeyEntry {
                                    key_namespace: FourCC::new(b"mdta"),
                                    key_value: "encoder".into()
                                },
                            ]
                        }
                        .into(),
                        Ilst {
                            name: None,
                            year: None,
                            covr: None,
                            desc: None,
                            ctoo: None,
                            cprt: None,
                            mdta: vec![
                                (
                                    1,
                                    IlstData {
                                        country_indicator: 0,
                                        language_indicator: 0,
                                        value: IlstDataValue::Utf8("iso6".into())
                                    }
                                ),
                                (
                                    2,
                                    IlstData {
                                        country_indicator: 0,
                                        language_indicator: 0,
                                        value: IlstDataValue::Utf8("512".into())
                                    }
                                ),
                                (
                                    3,
                                    IlstData {
                                        country_indicator: 0,
                                        language_indicator: 0,
                                        value: IlstDataValue::Utf8("iso6cmfcav01mp41".into())
                                    }
                                ),
                                (
                                    4,
                                    IlstData {
                                        country_indicator: 0,
                                        language_indicator: 0,
                                        value: IlstDataValue::Utf8("some value".into())
                                    }
                                ),
                                (
                                    5,
                                    IlstData {
                                        country_indicator: 0,
                                        language_indicator: 0,
                                        value: IlstDataValue::Utf8("Lavf63.1.101".into())
                                    }
                                )
                            ]
                        }
                        .into(),
                    ],
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    );

    // Make sure the av01 atom encodes/decodes to the exact same content.
    let av01 = &moov.trak[0].mdia.minf.stbl.stsd.codecs[0];
    av01.assert_encode_decode();

    let mut reencoded = Vec::new();
    ftyp.encode(&mut reencoded).expect("failed to encode ftyp");
    free.encode(&mut reencoded).expect("failed to encode free");
    mdat.encode(&mut reencoded).expect("failed to encode mdat");
    moov.encode(&mut reencoded).expect("failed to encode moov");

    let reencoded_buf = &mut reencoded.as_slice();
    let reencoded_ftyp = Ftyp::decode(reencoded_buf).expect("failed to re-decode ftyp");
    let reencoded_free = Free::decode(reencoded_buf).expect("failed to re-decode free");
    let reencoded_mdat = Mdat::decode(reencoded_buf).expect("failed to re-decode mdat");
    let reencoded_moov = Moov::decode(reencoded_buf).expect("failed to re-decode moov");
    assert_eq!(reencoded_ftyp, ftyp);
    assert_eq!(reencoded_free, free);
    assert_eq!(reencoded_mdat, mdat);
    assert_eq!(reencoded_moov, moov);

    // assert_eq!(buf, ENCODED);
}
