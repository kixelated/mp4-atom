use crate::*;

#[test]
fn bbb() {
    const ENCODED: &[u8] = include_bytes!("bbb.mp4");

    let mut buf = &mut std::io::Cursor::new(&ENCODED);

    let ftyp = Ftyp::decode(buf).expect("failed to decode ftyp");

    assert_eq!(
        ftyp,
        Ftyp {
            major_brand: b"iso6".into(),
            minor_version: 512,
            compatible_brands: vec![b"iso6".into(), b"cmfc".into(), b"mp41".into()],
        }
    );

    let moov = Moov::decode(buf).expect("failed to decode moov");
    assert_eq!(
        moov,
        Moov {
            mvhd: Mvhd {
                timescale: 1000,
                rate: 1.into(),
                volume: 1.into(),
                next_track_id: 2,
                ..Default::default()
            },
            mvex: Some(Mvex {
                trex: vec![
                    Trex {
                        track_id: 1,
                        default_sample_description_index: 1,
                        ..Default::default()
                    },
                    Trex {
                        track_id: 2,
                        default_sample_description_index: 1,
                        ..Default::default()
                    }
                ],
                ..Default::default()
            }),
            trak: vec![Trak {
                tkhd: Tkhd {
                    track_id: 1,
                    enabled: true,
                    width: 1280.into(),
                    height: 720.into(),
                    ..Default::default()
                },
                mdia: Mdia {
                    mdhd: Mdhd {
                        timescale: 24000,
                        language: "und".into(),
                        ..Default::default()
                    },
                    hdlr: Hdlr {
                        handler: b"vide".into(),
                        name: "(C) 2007 Google Inc. v08.13.2007.".into(),
                    },
                    minf: Minf {
                        smhd: None,
                        vmhd: Vmhd {
                            ..Default::default()
                        }
                        .into(),
                        dinf: Dinf {
                            dref: Dref {
                                urls: vec![Url {
                                    location: "".into(),
                                }],
                            },
                        },
                        stbl: Stbl {
                            stsd: Stsd {
                                codecs: vec![Avc1 {
                                    visual: Visual {
                                    data_reference_index: 1,
                                    width: 1280,
                                    height: 720,
                                    horizresolution: 72.into(),
                                    vertresolution: 72.into(),
                                    frame_count: 1,
                                    compressor: "".into(),
                                    depth: 24,
                                    },
                                    avcc: Avcc {
                                        configuration_version: 1,
                                        avc_profile_indication: 100,
                                        profile_compatibility: 0,
                                        avc_level_indication: 31,
                                        length_size: 4,
                                        sequence_parameter_sets: vec![b"gd\0\x1f\xac$\x84\x01@\x16\xec\x04@\0\0\x03\0@\0\0\x0c#\xc6\x0c\x92".into()],
                                        picture_parameter_sets:  vec![b"h\xee2\xc8\xb0".into()],
                                        ext: None,
                                    },
                                    btrt: Some(Btrt { buffer_size_db: 0, max_bitrate: 1991287, avg_bitrate: 1991287 }),
                                    colr: None,
                                    pasp: Some(Pasp {
                                        h_spacing: 1,
                                        v_spacing: 1,
                                    }),
                                    taic: None,
                                    fiel: None,
                                }
                                .into()],
                            },
                            stts: Stts {
                                ..Default::default()
                            },
                            stsc: Stsc {
                                ..Default::default()
                            },
                            stsz: Stsz {
                                ..Default::default()
                            },
                            stco: Some(Stco { ..Default::default() }),
                            ..Default::default()
                        },
                    },
                },
                ..Default::default()
            },
            Trak {
                tkhd: Tkhd {
                    track_id: 2,
                    alternate_group: 1,
                    enabled: true,
                    volume: 1.into(),
                    ..Default::default()
                },
                mdia: Mdia {
                    mdhd: Mdhd {
                        timescale: 44100,
                        language: "und".into(),
                        ..Default::default()
                    },
                    hdlr: Hdlr {
                        handler: b"soun".into(),
                        name: "(C) 2007 Google Inc. v08.13.2007.".into(),
                    },
                    minf: Minf {
                        smhd: Some(Smhd {
                            ..Default::default()
                        }),
                        dinf: Dinf {
                            dref: Dref {
                                urls: vec![Url {
                                    location: "".into(),
                                }],
                            },
                        },
                        stbl: Stbl {
                            stsd: Stsd {
                                codecs: vec![Mp4a {
                                    audio: Audio {
                                        data_reference_index: 1,
                                        channel_count: 2,
                                        sample_size: 16,
                                        sample_rate: 44100.into(),
                                    },
                                    esds: Esds {
                                        es_desc: esds::EsDescriptor {
                                            es_id: 2,
                                            dec_config: esds::DecoderConfig{
                                                object_type_indication: 64,
                                                stream_type: 5,
                                                max_bitrate: 125587,
                                                avg_bitrate: 125587,
                                                dec_specific: esds::DecoderSpecific {
                                                    profile: 2,
                                                    freq_index: 4,
                                                    chan_conf: 2,
                                                },
                                                ..Default::default()
                                            },
                                            sl_config: esds::SLConfig{},
                                        },
                                    },
                                    btrt: Some(Btrt { buffer_size_db: 0, max_bitrate: 125587, avg_bitrate: 125587 }),
                                    taic: None,
                                }
                                .into()],
                            },
                            stts: Stts {
                                ..Default::default()
                            },
                            stsc: Stsc {
                                ..Default::default()
                            },
                            stsz: Stsz {
                                ..Default::default()
                            },
                            stco: Some(Stco { ..Default::default() }),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                ..Default::default()
            }],
            udta: Some(Udta {
                meta: Some(Meta {
                    hdlr: Hdlr{ handler: FourCC::new(b"mdir"), name: "".into() },
                    items: vec![Ilst { name: None, year: None, covr: None, desc: None, ctoo: Some(Tool { country_indicator: 0, language_indicator: 0, text: "Lavf61.1.100".into()}) }.into(),],
                }),
                ..Default::default()
            }),

            ..Default::default()
        },
    );

    let moof = Moof::decode(&mut buf).expect("failed to decode moof");
    assert_eq!(
        moof,
        Moof {
            mfhd: Mfhd { sequence_number: 1 },
            traf: vec![Traf {
                tfhd: Tfhd {
                    track_id: 1,
                    sample_description_index: 1.into(),
                    default_sample_duration: 1000.into(),
                    default_sample_flags: 0x1010000.into(),
                    default_sample_size: 215.into(),
                    ..Default::default()
                },
                tfdt: Some(Tfdt {
                    ..Default::default()
                }),
                trun: vec![Trun {
                    data_offset: 116.into(),
                    entries: vec![TrunEntry {
                        flags: Some(33554432),
                        ..Default::default()
                    }],
                }],
                sbgp: vec![],
                sgpd: vec![],
                subs: vec![],
                saiz: vec![],
                saio: vec![],
                meta: None,
                senc: None,
                udta: None,
            }],
        },
    );

    let mdat = Mdat::decode(&mut buf).expect("failed to decode mdat");
    assert_eq!(
        mdat,
        Mdat {
            data: vec![
                0x00, 0x00, 0x00, 0xD3, 0x65, 0x88, 0x80, 0x80, 0x03, 0x3F, 0xFE, 0xF5, 0xF8, 0x45,
                0x4F, 0x32, 0xCB, 0x1B, 0xB4, 0x20, 0x3F, 0x85, 0x4D, 0xD6, 0x9B, 0xC2, 0xCA, 0x91,
                0xB2, 0xBC, 0xE1, 0xFB, 0x35, 0x27, 0x44, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00,
                0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x50, 0x99, 0x98, 0x41, 0xD1, 0xAF, 0xD3, 0x24,
                0xAE, 0xA0, 0x00, 0x00, 0x03, 0x00, 0x00, 0x0F, 0x60, 0x00, 0x11, 0xC0, 0x00, 0x1B,
                0x40, 0x00, 0x4E, 0x40, 0x01, 0x1F, 0x00, 0x03, 0xB8, 0x00, 0x10, 0x80, 0x00, 0x59,
                0x00, 0x02, 0x38, 0x00, 0x0B, 0xE0, 0x00, 0x5E, 0x00, 0x02, 0x20, 0x00, 0x11, 0x00,
                0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
                0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00,
                0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00,
                0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
                0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00,
                0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00,
                0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
                0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00,
                0x03, 0x00, 0x00, 0x40, 0x41,
            ],
        }
    );

    let moof = Moof::decode(&mut buf).expect("failed to decode moof");
    assert_eq!(
        moof,
        Moof {
            mfhd: Mfhd { sequence_number: 2 },
            traf: vec![Traf {
                tfhd: Tfhd {
                    track_id: 2,
                    sample_description_index: 1.into(),
                    default_sample_duration: 1024.into(),
                    default_sample_flags: 0x2000000.into(),
                    default_sample_size: 9.into(),
                    ..Default::default()
                },
                tfdt: Some(Tfdt {
                    ..Default::default()
                }),
                trun: vec![Trun {
                    data_offset: 112.into(),
                    entries: vec![Default::default()],
                }],
                sbgp: vec![],
                sgpd: vec![],
                subs: vec![],
                saiz: vec![],
                saio: vec![],
                meta: None,
                senc: None,
                udta: None,
            }],
        },
    );

    let mdat = Mdat::decode(&mut buf).expect("failed to decode mdat");
    assert_eq!(
        mdat,
        Mdat {
            data: vec![0x21, 0x00, 0x49, 0x90, 0x02, 0x19, 0x00, 0x23, 0x80],
        }
    );

    let mut buf = Vec::new();

    ftyp.encode(&mut buf).expect("failed to encode ftyp");
    moov.encode(&mut buf).expect("failed to encode moov");
    moof.encode(&mut buf).expect("failed to encode moof");
    mdat.encode(&mut buf).expect("failed to encode mdat");

    // One day:
    // assert_eq!(buf, ENCODED);
}
