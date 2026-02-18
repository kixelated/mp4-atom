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
                                        compressor: "AVC Coding".into(),
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
                        ..Default::default()
                    },
                },
                senc: None,
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
                        ..Default::default()
                    },
                },
                senc: None,
                udta: None,
            },
        ],
        udta: Some(Udta::default()),
        ..Default::default()
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
                senc: Some(Senc {
                    version: SencBoxVersion::V0,
                    use_subsamples: true,
                    data: vec![
                        0, 0, 0, 234, 0, 1, 0, 129, 0, 0, 46, 38, 0, 1, 0, 114, 0, 0, 2, 48, 0, 1,
                        0, 114, 0, 0, 4, 81, 0, 1, 0, 114, 0, 0, 3, 238, 0, 1, 0, 114, 0, 0, 3,
                        214, 0, 1, 0, 114, 0, 0, 4, 129, 0, 1, 0, 114, 0, 0, 3, 219, 0, 1, 0, 114,
                        0, 0, 5, 64, 0, 1, 0, 114, 0, 0, 5, 71, 0, 1, 0, 114, 0, 0, 4, 203, 0, 1,
                        0, 114, 0, 0, 5, 182, 0, 1, 0, 114, 0, 0, 5, 10, 0, 1, 0, 114, 0, 0, 5, 50,
                        0, 1, 0, 114, 0, 0, 4, 113, 0, 1, 0, 114, 0, 0, 3, 105, 0, 1, 0, 114, 0, 0,
                        3, 138, 0, 1, 0, 114, 0, 0, 3, 107, 0, 1, 0, 114, 0, 0, 2, 227, 0, 1, 0,
                        114, 0, 0, 3, 85, 0, 1, 0, 114, 0, 0, 4, 6, 0, 1, 0, 114, 0, 0, 4, 105, 0,
                        1, 0, 114, 0, 0, 4, 26, 0, 1, 0, 114, 0, 0, 5, 32, 0, 1, 0, 114, 0, 0, 6,
                        24, 0, 1, 0, 114, 0, 0, 5, 193, 0, 1, 0, 114, 0, 0, 5, 73, 0, 1, 0, 114, 0,
                        0, 4, 179, 0, 1, 0, 114, 0, 0, 4, 249, 0, 1, 0, 114, 0, 0, 4, 215, 0, 1, 0,
                        114, 0, 0, 6, 79, 0, 1, 0, 114, 0, 0, 4, 137, 0, 1, 0, 114, 0, 0, 4, 131,
                        0, 1, 0, 114, 0, 0, 4, 126, 0, 1, 0, 114, 0, 0, 4, 87, 0, 1, 0, 114, 0, 0,
                        4, 164, 0, 1, 0, 114, 0, 0, 5, 75, 0, 1, 0, 114, 0, 0, 4, 96, 0, 1, 0, 114,
                        0, 0, 4, 97, 0, 1, 0, 114, 0, 0, 4, 123, 0, 1, 0, 114, 0, 0, 3, 251, 0, 1,
                        0, 114, 0, 0, 5, 51, 0, 1, 0, 114, 0, 0, 4, 4, 0, 1, 0, 114, 0, 0, 6, 91,
                        0, 1, 0, 114, 0, 0, 3, 83, 0, 1, 0, 114, 0, 0, 4, 92, 0, 1, 0, 114, 0, 0,
                        5, 47, 0, 1, 0, 114, 0, 0, 4, 185, 0, 1, 0, 114, 0, 0, 6, 63, 0, 1, 0, 114,
                        0, 0, 4, 40, 0, 1, 0, 114, 0, 0, 5, 80, 0, 1, 0, 114, 0, 0, 5, 124, 0, 1,
                        0, 114, 0, 0, 4, 236, 0, 1, 0, 114, 0, 0, 4, 73, 0, 1, 0, 114, 0, 0, 4,
                        240, 0, 1, 0, 114, 0, 0, 5, 113, 0, 1, 0, 114, 0, 0, 3, 183, 0, 1, 0, 114,
                        0, 0, 2, 229, 0, 1, 0, 114, 0, 0, 3, 64, 0, 1, 0, 114, 0, 0, 3, 77, 0, 1,
                        0, 114, 0, 0, 3, 161, 0, 1, 0, 114, 0, 0, 3, 39, 0, 1, 0, 114, 0, 0, 4, 42,
                        0, 1, 0, 114, 0, 0, 3, 229, 0, 1, 0, 114, 0, 0, 3, 180, 0, 1, 0, 114, 0, 0,
                        4, 27, 0, 1, 0, 128, 0, 0, 17, 95, 0, 1, 0, 117, 0, 0, 4, 201, 0, 1, 0,
                        119, 0, 0, 3, 140, 0, 1, 0, 120, 0, 0, 4, 205, 0, 1, 0, 121, 0, 0, 3, 150,
                        0, 1, 0, 120, 0, 0, 3, 187, 0, 1, 0, 120, 0, 0, 3, 114, 0, 1, 0, 119, 0, 0,
                        3, 208, 0, 1, 0, 120, 0, 0, 4, 251, 0, 1, 0, 120, 0, 0, 3, 137, 0, 1, 0,
                        119, 0, 0, 3, 120, 0, 1, 0, 120, 0, 0, 4, 137, 0, 1, 0, 121, 0, 0, 3, 15,
                        0, 1, 0, 115, 0, 0, 3, 125, 0, 1, 0, 120, 0, 0, 3, 122, 0, 1, 0, 119, 0, 0,
                        3, 56, 0, 1, 0, 119, 0, 0, 5, 169, 0, 1, 0, 119, 0, 0, 3, 210, 0, 1, 0,
                        120, 0, 0, 3, 195, 0, 1, 0, 120, 0, 0, 6, 31, 0, 1, 0, 119, 0, 0, 4, 222,
                        0, 1, 0, 120, 0, 0, 4, 29, 0, 1, 0, 119, 0, 0, 4, 81, 0, 1, 0, 115, 0, 0,
                        3, 101, 0, 1, 0, 114, 0, 0, 2, 137, 0, 1, 0, 114, 0, 0, 2, 169, 0, 1, 0,
                        114, 0, 0, 3, 98, 0, 1, 0, 114, 0, 0, 2, 234, 0, 1, 0, 114, 0, 0, 2, 238,
                        0, 1, 0, 114, 0, 0, 3, 211, 0, 1, 0, 114, 0, 0, 2, 232, 0, 1, 0, 114, 0, 0,
                        4, 40, 0, 1, 0, 114, 0, 0, 2, 150, 0, 1, 0, 114, 0, 0, 2, 205, 0, 1, 0,
                        114, 0, 0, 3, 35, 0, 1, 0, 114, 0, 0, 4, 6, 0, 1, 0, 114, 0, 0, 3, 148, 0,
                        1, 0, 114, 0, 0, 3, 122, 0, 1, 0, 114, 0, 0, 3, 205, 0, 1, 0, 114, 0, 0, 4,
                        215, 0, 1, 0, 114, 0, 0, 3, 46, 0, 1, 0, 114, 0, 0, 4, 228, 0, 1, 0, 114,
                        0, 0, 4, 29, 0, 1, 0, 114, 0, 0, 4, 178, 0, 1, 0, 114, 0, 0, 4, 122, 0, 1,
                        0, 114, 0, 0, 4, 138, 0, 1, 0, 114, 0, 0, 4, 103, 0, 1, 0, 114, 0, 0, 4,
                        134, 0, 1, 0, 114, 0, 0, 5, 15, 0, 1, 0, 114, 0, 0, 3, 191, 0, 1, 0, 114,
                        0, 0, 5, 72, 0, 1, 0, 114, 0, 0, 3, 204, 0, 1, 0, 129, 0, 0, 24, 108, 0, 1,
                        0, 114, 0, 0, 2, 58, 0, 1, 0, 114, 0, 0, 3, 12, 0, 1, 0, 114, 0, 0, 3, 174,
                        0, 1, 0, 114, 0, 0, 3, 209, 0, 1, 0, 114, 0, 0, 3, 36, 0, 1, 0, 114, 0, 0,
                        3, 165, 0, 1, 0, 114, 0, 0, 3, 175, 0, 1, 0, 114, 0, 0, 3, 197, 0, 1, 0,
                        114, 0, 0, 3, 54, 0, 1, 0, 127, 0, 0, 37, 174, 0, 1, 0, 114, 0, 0, 6, 13,
                        0, 1, 0, 115, 0, 0, 6, 140, 0, 1, 0, 115, 0, 0, 6, 131, 0, 1, 0, 114, 0, 0,
                        8, 221, 0, 1, 0, 114, 0, 0, 7, 66, 0, 1, 0, 114, 0, 0, 5, 119, 0, 1, 0,
                        114, 0, 0, 5, 92, 0, 1, 0, 114, 0, 0, 5, 39, 0, 1, 0, 114, 0, 0, 4, 80, 0,
                        1, 0, 114, 0, 0, 4, 103, 0, 1, 0, 114, 0, 0, 6, 84, 0, 1, 0, 114, 0, 0, 4,
                        235, 0, 1, 0, 114, 0, 0, 4, 246, 0, 1, 0, 114, 0, 0, 5, 249, 0, 1, 0, 114,
                        0, 0, 5, 73, 0, 1, 0, 114, 0, 0, 4, 160, 0, 1, 0, 114, 0, 0, 5, 71, 0, 1,
                        0, 114, 0, 0, 4, 179, 0, 1, 0, 114, 0, 0, 6, 42, 0, 1, 0, 114, 0, 0, 5,
                        212, 0, 1, 0, 114, 0, 0, 5, 103, 0, 1, 0, 114, 0, 0, 5, 208, 0, 1, 0, 114,
                        0, 0, 6, 37, 0, 1, 0, 114, 0, 0, 6, 162, 0, 1, 0, 114, 0, 0, 5, 26, 0, 1,
                        0, 114, 0, 0, 6, 163, 0, 1, 0, 114, 0, 0, 6, 75, 0, 1, 0, 114, 0, 0, 5,
                        138, 0, 1, 0, 114, 0, 0, 6, 235, 0, 1, 0, 114, 0, 0, 5, 241, 0, 1, 0, 114,
                        0, 0, 5, 30, 0, 1, 0, 114, 0, 0, 5, 216, 0, 1, 0, 114, 0, 0, 5, 116, 0, 1,
                        0, 114, 0, 0, 4, 156, 0, 1, 0, 114, 0, 0, 6, 129, 0, 1, 0, 114, 0, 0, 3,
                        238, 0, 1, 0, 114, 0, 0, 4, 34, 0, 1, 0, 114, 0, 0, 4, 226, 0, 1, 0, 114,
                        0, 0, 5, 56, 0, 1, 0, 114, 0, 0, 5, 53, 0, 1, 0, 114, 0, 0, 6, 35, 0, 1, 0,
                        114, 0, 0, 6, 190, 0, 1, 0, 114, 0, 0, 5, 20, 0, 1, 0, 114, 0, 0, 7, 73, 0,
                        1, 0, 114, 0, 0, 8, 216, 0, 1, 0, 114, 0, 0, 5, 163, 0, 1, 0, 114, 0, 0, 5,
                        74, 0, 1, 0, 114, 0, 0, 5, 115, 0, 1, 0, 114, 0, 0, 5, 125, 0, 1, 0, 114,
                        0, 0, 6, 133, 0, 1, 0, 114, 0, 0, 5, 89, 0, 1, 0, 114, 0, 0, 5, 129, 0, 1,
                        0, 114, 0, 0, 5, 62, 0, 1, 0, 114, 0, 0, 5, 64, 0, 1, 0, 114, 0, 0, 4, 252,
                        0, 1, 0, 114, 0, 0, 5, 47, 0, 1, 0, 114, 0, 0, 5, 4, 0, 1, 0, 114, 0, 0, 5,
                        27, 0, 1, 0, 114, 0, 0, 4, 250, 0, 1, 0, 114, 0, 0, 5, 86, 0, 1, 0, 114, 0,
                        0, 4, 228, 0, 1, 0, 114, 0, 0, 5, 4, 0, 1, 0, 114, 0, 0, 4, 231, 0, 1, 0,
                        114, 0, 0, 4, 242, 0, 1, 0, 114, 0, 0, 5, 29, 0, 1, 0, 114, 0, 0, 4, 233,
                        0, 1, 0, 114, 0, 0, 4, 180, 0, 1, 0, 114, 0, 0, 4, 250, 0, 1, 0, 114, 0, 0,
                        4, 15, 0, 1, 0, 114, 0, 0, 4, 110, 0, 1, 0, 114, 0, 0, 4, 222, 0, 1, 0,
                        114, 0, 0, 4, 53, 0, 1, 0, 114, 0, 0, 5, 69, 0, 1, 0, 114, 0, 0, 5, 68, 0,
                        1, 0, 114, 0, 0, 5, 69, 0, 1, 0, 114, 0, 0, 5, 49, 0, 1, 0, 114, 0, 0, 5,
                        9, 0, 1, 0, 114, 0, 0, 4, 89, 0, 1, 0, 114, 0, 0, 4, 214, 0, 1, 0, 114, 0,
                        0, 5, 90, 0, 1, 0, 114, 0, 0, 5, 152, 0, 1, 0, 114, 0, 0, 5, 175, 0, 1, 0,
                        114, 0, 0, 5, 118, 0, 1, 0, 114, 0, 0, 5, 138, 0, 1, 0, 114, 0, 0, 5, 70,
                        0, 1, 0, 114, 0, 0, 5, 125, 0, 1, 0, 114, 0, 0, 5, 6, 0, 1, 0, 114, 0, 0,
                        4, 255, 0, 1, 0, 114, 0, 0, 4, 180, 0, 1, 0, 114, 0, 0, 5, 107, 0, 1, 0,
                        114, 0, 0, 5, 228, 0, 1, 0, 114, 0, 0, 7, 226, 0, 1, 0, 114, 0, 0, 6, 20,
                        0, 1, 0, 114, 0, 0, 5, 88, 0, 1, 0, 114, 0, 0, 5, 234, 0, 1, 0, 114, 0, 0,
                        5, 84, 0, 1, 0, 114, 0, 0, 5, 15, 0, 1, 0, 114, 0, 0, 4, 217, 0, 1, 0, 114,
                        0, 0, 6, 70, 0, 1, 0, 114, 0, 0, 5, 102, 0, 1, 0, 114, 0, 0, 4, 217, 0, 1,
                        0, 114, 0, 0, 6, 13, 0, 1, 0, 114, 0, 0, 5, 161, 0, 1, 0, 114, 0, 0, 6, 3,
                        0, 1, 0, 119, 0, 0, 6, 60, 0, 1, 0, 114, 0, 0, 3, 59
                    ]
                }),
                udta: None,
            }]
        }
    );

    assert_eq!(0, buf.remaining());
}
