use crate::*;

#[test]
fn esds() {
    const ENCODED: &[u8] = include_bytes!("esds.mp4");

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
                        timescale: 12288,
                        language: "und".into(),
                        ..Default::default()
                    },
                    hdlr: Hdlr {
                        handler: b"vide".into(),
                        name: "(C) 2007 Google Inc. v08.13.2007.".into(),
                    },
                    minf: Minf {
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
                                    compressor: "\u{15}Lavc60.31.102 libx264".into(),
                                    depth: 24,
                                    },
                                    avcc: Avcc {
                                        configuration_version: 1,
                                        avc_profile_indication: 66,
                                        profile_compatibility: 192,
                                        avc_level_indication: 31,
                                        length_size: 4,
                                        sequence_parameter_sets: vec![b"gB\xc0\x1f\xda\x01@\x16\xec\x04@\0\0\x03\0@\0\0\x0c#\xc6\x0c\xa8".into()],
                                        picture_parameter_sets:  vec![b"h\xce\x0f\xc8".into()],
                                        ext: None,
                                    },
                                    btrt: None,
                                    colr: None,
                                    pasp: Some(Pasp {
                                        h_spacing: 1,
                                        v_spacing: 1
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
                        ..Default::default()
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
                                                max_bitrate: 128000,
                                                avg_bitrate: 128000,
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
                                    btrt: Some(Btrt { buffer_size_db: 0, max_bitrate: 128000, avg_bitrate: 128000 }),
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
            udta: Some(Udta::default()),
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
                    default_sample_duration: 512.into(),
                    default_sample_flags: 0x1010000.into(),
                    default_sample_size: 3377.into(),
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

    let mut buf = Vec::new();
    ftyp.encode(&mut buf).expect("failed to encode ftyp");
    moov.encode(&mut buf).expect("failed to encode moov");
    moof.encode(&mut buf).expect("failed to encode moof");

    // One day:
    // assert_eq!(buf, ENCODED);
}
