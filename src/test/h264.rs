use crate::*;

// Contains additional AVCC fields that seem to be somewhat optional.
#[test]
fn avcc_ext() {
    const ENCODED: &[u8] = include_bytes!("h264.mp4");

    let buf = &mut std::io::Cursor::new(&ENCODED);

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
    let expected = Moov {
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
                w: 1073741824,
            },
            next_track_id: 2,
        },
        meta: None,
        mvex: Some(Mvex {
            mehd: None,
            trex: vec![
                Trex {
                    track_id: 1,
                    default_sample_description_index: 1,
                    default_sample_duration: 0,
                    default_sample_size: 0,
                    default_sample_flags: 0,
                },
                Trex {
                    track_id: 2,
                    default_sample_description_index: 1,
                    default_sample_duration: 0,
                    default_sample_size: 0,
                    default_sample_flags: 0,
                },
            ],
        }),
        trak: vec![
            Trak {
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
                        w: 1073741824,
                    },
                    width: 1280.into(),
                    height: 720.into(),
                },
                edts: None,
                meta: None,
                mdia: Mdia {
                    mdhd: Mdhd {
                        creation_time: 0,
                        modification_time: 0,
                        timescale: 15360,
                        duration: 0,
                        language: "und".into(),
                    },
                    hdlr: Hdlr {
                        handler: b"vide".into(),
                        name: "L-SMASH Video Handler".to_string(),
                    },
                    minf: Minf {
                        vmhd: Some(Vmhd {
                            graphics_mode: 0,
                            op_color: RgbColor {
                                red: 0,
                                green: 0,
                                blue: 0,
                            },
                        }),
                        smhd: None,
                        dinf: Dinf {
                            dref: Dref {
                                urls: vec![Url::default()],
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
                                        compressor: "\nAVC Coding".into(),
                                        depth: 24,
                                    },
                                    avcc: Avcc {
                                        configuration_version: 1,
                                        avc_profile_indication: 100,
                                        profile_compatibility: 0,
                                        avc_level_indication: 31,
                                        length_size: 4,
                                        sequence_parameter_sets: vec![vec![
                                            103, 100, 0, 31, 172, 217, 128, 80, 5, 187, 1, 106, 2,
                                            2, 2, 128, 0, 0, 3, 0, 128, 0, 0, 30, 7, 140, 24, 205,
                                        ]],
                                        picture_parameter_sets: vec![vec![104, 233, 123, 44, 139]],
                                        ext: Some(AvccExt {
                                            chroma_format: 1,
                                            bit_depth_luma: 8,
                                            bit_depth_chroma: 8,
                                            sequence_parameter_sets_ext: vec![],
                                        }),
                                    },
                                    btrt: Some(Btrt {
                                        buffer_size_db: 0,
                                        max_bitrate: 2453499,
                                        avg_bitrate: 2453499,
                                    }),
                                    colr: Some(Colr::Nclx {
                                        colour_primaries: 1,
                                        transfer_characteristics: 1,
                                        matrix_coefficients: 1,
                                        full_range_flag: false,
                                    }),
                                    pasp: Some(Pasp {
                                        h_spacing: 1,
                                        v_spacing: 1,
                                    }),
                                    taic: None,
                                    fiel: None,
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
                    },
                },
                udta: None,
            },
            Trak {
                tkhd: Tkhd {
                    creation_time: 0,
                    modification_time: 0,
                    track_id: 2,
                    duration: 0,
                    layer: 0,
                    alternate_group: 1,
                    enabled: true,
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
                        w: 1073741824,
                    },
                    width: 0.into(),
                    height: 0.into(),
                },
                edts: None,
                meta: None,
                mdia: Mdia {
                    mdhd: Mdhd {
                        creation_time: 0,
                        modification_time: 0,
                        timescale: 48000,
                        duration: 0,
                        language: "und".into(),
                    },
                    hdlr: Hdlr {
                        handler: b"soun".into(),
                        name: "L-SMASH Audio Handler".into(),
                    },
                    minf: Minf {
                        vmhd: None,
                        smhd: Some(Smhd::default()),
                        dinf: Dinf {
                            dref: Dref {
                                urls: vec![Url::default()],
                            },
                        },
                        stbl: Stbl {
                            stsd: Stsd {
                                codecs: vec![Mp4a {
                                    audio: Audio {
                                        data_reference_index: 1,
                                        channel_count: 2,
                                        sample_size: 16,
                                        sample_rate: 48000.into(),
                                    },
                                    esds: Esds {
                                        es_desc: esds::EsDescriptor {
                                            es_id: 2,
                                            dec_config: esds::DecoderConfig {
                                                object_type_indication: 64,
                                                stream_type: 5,
                                                up_stream: 0,
                                                buffer_size_db: u24::default(),
                                                max_bitrate: 160000,
                                                avg_bitrate: 160000,
                                                dec_specific: esds::DecoderSpecific {
                                                    profile: 2,
                                                    freq_index: 3,
                                                    chan_conf: 2,
                                                },
                                            },
                                            sl_config: esds::SLConfig::default(),
                                        },
                                    },
                                    btrt: Some(Btrt {
                                        buffer_size_db: 0,
                                        max_bitrate: 160000,
                                        avg_bitrate: 160000,
                                    }),
                                    taic: None,
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
                    },
                },
                udta: None,
            },
        ],
        udta: Some(Udta { meta: None }),
    };

    assert_eq!(moov, expected, "different decoded result");

    // Make sure the avc1 atom encodes/decodes to the exact same content.
    let avc1 = &moov.trak[0].mdia.minf.stbl.stsd.codecs[0];
    avc1.assert_encode_decode();

    let mut buf = Vec::new();
    ftyp.encode(&mut buf).expect("failed to encode ftyp");
    moov.encode(&mut buf).expect("failed to encode moov");

    // assert_eq!(buf, ENCODED);
}

#[test]
fn avcc_ext_2() {
    const ENCODED: &[u8] = include_bytes!("h264_avcc_ext_2.mp4");

    let buf = &mut std::io::Cursor::new(&ENCODED);

    let ftyp = Ftyp::decode(buf).expect("failed to decode ftyp");

    assert_eq!(
        ftyp,
        Ftyp {
            major_brand: b"iso6".into(),
            minor_version: 512,
            compatible_brands: vec![b"iso6".into(), b"cmfc".into(), b"mp41".into()],
        }
    );

    // This was failing because the avcc atom was under decoded.
    let _ = Moov::decode(buf).expect("failed to decode moov");
}

#[test]
fn avc_encrypted_segment() {
    // mdat + trun removed to reduce size.
    const ENCODED: &[u8] = include_bytes!("h264_encrypted_segment.mp4");

    let buf = &mut std::io::Cursor::new(&ENCODED);

    let styp = match Any::decode(buf) {
        Ok(Any::Styp(styp)) => styp,
        Ok(atom) => panic!("unexpected {} while decoding any styp", atom.kind()),
        Err(e) => panic!("failed to decode styp with error: {e}"),
    };
    assert_eq!(
        styp,
        Styp {
            major_brand: b"msdh".into(),
            minor_version: 0,
            compatible_brands: vec![b"msix".into(), b"msdh".into(), b"dash".into()]
        }
    );

    let sidx = match Any::decode(buf) {
        Ok(Any::Sidx(sidx)) => sidx,
        Ok(atom) => panic!("unexpected {} while decoding any sidx", atom.kind()),
        Err(e) => panic!("failed to decode sidx with error: {e}"),
    };
    assert_eq!(
        sidx,
        Sidx {
            reference_id: 1,
            timescale: 10000000,
            earliest_presentation_time: 342202323002237,
            first_offset: 32,
            references: vec![SegmentReference {
                reference_type: false,
                reference_size: 345327,
                subsegment_duration: 78078000,
                starts_with_sap: true,
                sap_type: 0,
                sap_delta_time: 0
            }]
        }
    );

    let prft = match Any::decode(buf) {
        Ok(Any::Prft(prft)) => prft,
        Ok(atom) => panic!("unexpected {} while decoding any prft", atom.kind()),
        Err(e) => panic!("failed to decode prft with error: {e}"),
    };
    assert_eq!(
        prft,
        Prft {
            reference_track_id: 1,
            ntp_timestamp: 17013886056065052993,
            media_time: 342202323002237,
            utc_time_semantics: ReferenceTime::Input,
        }
    );

    let moof = match Any::decode(buf) {
        Ok(Any::Moof(moof)) => moof,
        Ok(atom) => panic!("unexpected {} while decoding any moof", atom.kind()),
        Err(e) => panic!("failed to decode moof with error: {e}"),
    };
    assert_eq!(
        moof,
        Moof {
            mfhd: Mfhd {
                sequence_number: 4382715
            },
            traf: vec![Traf {
                tfhd: Tfhd {
                    track_id: 1,
                    base_data_offset: None,
                    sample_description_index: None,
                    default_sample_duration: None,
                    default_sample_size: None,
                    default_sample_flags: None,
                },
                tfdt: Some(Tfdt {
                    base_media_decode_time: 342202323002237
                }),
                trun: vec![],
                sbgp: vec![],
                sgpd: vec![],
                subs: vec![],
                saiz: vec![Saiz {
                    aux_info: Some(AuxInfo {
                        aux_info_type: b"cbcs".into(),
                        aux_info_type_parameter: 0
                    }),
                    default_sample_info_size: 8,
                    sample_count: 234,
                    sample_info_size: vec![],
                }],
                saio: vec![Saio {
                    aux_info: Some(AuxInfo {
                        aux_info_type: b"cbcs".into(),
                        aux_info_type_parameter: 0
                    }),
                    offsets: vec![3901],
                }],
                meta: None,
                udta: None,
            }]
        }
    );

    assert_eq!(0, buf.remaining());
}
