use crate::*;

#[test]
fn av1() {
    const ENCODED: &[u8] = include_bytes!("av1.mp4");

    let buf = &mut std::io::Cursor::new(ENCODED);
    let ftyp = Ftyp::decode(buf).expect("failed to decode ftyp");

    assert_eq!(
        ftyp,
        Ftyp {
            major_brand: b"iso6".into(),
            minor_version: 512,
            compatible_brands: vec![
                b"iso6".into(),
                b"cmfc".into(),
                b"av01".into(),
                b"mp41".into()
            ],
        }
    );

    let moov = Moov::decode(buf).expect("failed to decode moov");
    assert_eq!(
        moov,
        Moov {
            mvhd: Mvhd {
                creation_time: 0,
                modification_time: 0,
                timescale: 1000,
                duration: 0,
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
            mvex: Some(Mvex {
                mehd: None,
                trex: vec![Trex {
                    track_id: 1,
                    default_sample_description_index: 1,
                    default_sample_duration: 0,
                    default_sample_size: 0,
                    default_sample_flags: 0
                }]
            }),
            trak: vec![Trak {
                tkhd: Tkhd {
                    creation_time: 0,
                    modification_time: 0,
                    track_id: 1,
                    duration: 0,
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
                    width: 1920.into(),
                    height: 1080.into()
                },
                edts: None,
                meta: None,
                mdia: Mdia {
                    mdhd: Mdhd {
                        creation_time: 0,
                        modification_time: 0,
                        timescale: 25000,
                        duration: 0,
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
                                        max_bitrate: 914496,
                                        avg_bitrate: 914496
                                    }),
                                    ..Default::default()
                                }
                                .into()],
                            },
                            stts: Stts::default(),
                            ctts: None,
                            stss: None,
                            stsc: Stsc::default(),
                            stsz: Stsz::default(),
                            stco: Some(Stco::default()),
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
                senc: None,
                udta: None,
            }],
            udta: Some(Udta {
                meta: Some(Meta {
                    hdlr: Hdlr {
                        handler: FourCC::new(b"mdir"),
                        name: "".into()
                    },
                    items: vec![Ilst {
                        name: None,
                        year: None,
                        covr: None,
                        desc: None,
                        ctoo: Some(Tool {
                            country_indicator: 0,
                            language_indicator: 0,
                            text: "Lavf61.7.100".into()
                        })
                    }
                    .into(),],
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    );

    let moof = Moof::decode(buf).expect("failed to decode moof");
    assert_eq!(
        moof,
        Moof {
            mfhd: Mfhd { sequence_number: 1 },
            traf: vec![Traf {
                tfhd: Tfhd {
                    track_id: 1,
                    base_data_offset: None,
                    sample_description_index: Some(1),
                    default_sample_duration: Some(1000),
                    default_sample_size: Some(252),
                    default_sample_flags: Some(16842752)
                },
                tfdt: Some(Tfdt {
                    base_media_decode_time: 0
                }),
                trun: vec![Trun {
                    data_offset: Some(116),
                    entries: vec![TrunEntry {
                        duration: None,
                        size: None,
                        flags: Some(33554432),
                        cts: None
                    }]
                }],
                sbgp: vec![],
                sgpd: vec![],
                subs: vec![],
                saiz: vec![],
                saio: vec![],
                meta: None,
                senc: None,
                udta: None,
            }]
        }
    );

    // Make sure the av01 atom encodes/decodes to the exact same content.
    let av01 = &moov.trak[0].mdia.minf.stbl.stsd.codecs[0];
    av01.assert_encode_decode();

    let mut buf = Vec::new();
    ftyp.encode(&mut buf).expect("failed to encode ftyp");
    moov.encode(&mut buf).expect("failed to encode moov");
    moof.encode(&mut buf).expect("failed to encode moof");

    // assert_eq!(buf, ENCODED);
}
