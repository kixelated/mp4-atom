use crate::*;

#[test]
fn mebx() {
    // A real two-track capture: an HEVC video track plus a multiplexed/"boxed"
    // timed metadata track (`mebx`, ISO/IEC 14496-12:2026 Section 12.8) using
    // the `me4c` key namespace, referencing the video track via a `cdsc`
    // track reference. Declares one key per colour-calibration value plus a
    // `labl` key repeated with two different `loca` variants (ISO/IEC
    // 14496-12:2026 Section 12.8.4.5 explicitly allows this: several keys of
    // the same key type, differing only by locale). The source file also
    // carried an `iods` box in `moov`, which this crate doesn't support (an
    // unrelated legacy MPEG-4 object descriptor box); it was stripped from
    // the fixture, with `moov`'s size and `stco` offsets adjusted to match.
    //
    // This example was taken from:
    // https://mpeggroup.github.io/FileFormatConformance/files/under_consideration/mebx/test_mebx_me4c.mp4
    const ENCODED: &[u8] = include_bytes!("mebx.mp4");

    let buf = &mut std::io::Cursor::new(ENCODED);
    let ftyp = Ftyp::decode(buf).expect("failed to decode ftyp");

    assert_eq!(
        ftyp,
        Ftyp {
            major_brand: b"mp42".into(),
            minor_version: 0,
            compatible_brands: vec![b"mp42".into(), b"isom".into()],
        }
    );

    let moov = Moov::decode(buf).expect("failed to decode moov");

    fn key(local_key_id: &[u8; 4], setu: &str, loca: Option<&str>) -> MebxKey {
        MebxKey {
            local_key_id: FourCC::new(local_key_id),
            keyd: Keyd {
                key_namespace: FourCC::new(b"me4c"),
                key_value: local_key_id.to_vec(),
            },
            loca: loca.map(|locale| Loca {
                locale: locale.into(),
            }),
            setu: Some(Setu {
                data: setu.as_bytes().to_vec(),
            }),
        }
    }

    assert_eq!(
        moov,
        Moov {
            mvhd: Mvhd {
                creation_time: 3732765738,
                modification_time: 3732765738,
                timescale: 600,
                duration: 600,
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
                next_track_id: 3
            },
            trak: vec![
                Trak {
                    tkhd: Tkhd {
                        creation_time: 3732765738,
                        modification_time: 3732765738,
                        track_id: 1,
                        duration: 600,
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
                        width: 64.into(),
                        height: 48.into(),
                    },
                    mdia: Mdia {
                        mdhd: Mdhd {
                            creation_time: 3732765738,
                            modification_time: 3732765738,
                            timescale: 30000,
                            duration: 30000,
                            language: "und".into(),
                        },
                        hdlr: Hdlr {
                            handler: FourCC::new(b"vide"),
                            name: "vide".into(),
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
                                    codecs: vec![Hvc1 {
                                        visual: Visual {
                                            data_reference_index: 1,
                                            width: 64,
                                            height: 48,
                                            horizresolution: 72.into(),
                                            vertresolution: 72.into(),
                                            frame_count: 1,
                                            compressor: "".into(),
                                            depth: 24,
                                        },
                                        hvcc: Hvcc {
                                            configuration_version: 1,
                                            general_profile_space: 0,
                                            general_tier_flag: false,
                                            general_profile_idc: 4,
                                            general_profile_compatibility_flags: [8, 0, 0, 0],
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
                                            length_size_minus_one: 0,
                                            arrays: vec![
                                                HvcCArray {
                                                    completeness: true,
                                                    nal_unit_type: 32,
                                                    nalus: vec![vec![
                                                        64, 1, 12, 1, 255, 255, 4, 8, 0, 0, 3, 0,
                                                        159, 168, 0, 0, 3, 0, 0, 30, 186, 2, 64
                                                    ]]
                                                },
                                                HvcCArray {
                                                    completeness: true,
                                                    nal_unit_type: 33,
                                                    nalus: vec![vec![
                                                        66, 1, 1, 4, 8, 0, 0, 3, 0, 159, 168, 0, 0,
                                                        3, 0, 0, 30, 160, 32, 131, 22, 91, 171,
                                                        147, 43, 154, 2, 0, 0, 3, 0, 2, 0, 0, 3, 0,
                                                        50, 16
                                                    ]]
                                                },
                                                HvcCArray {
                                                    completeness: true,
                                                    nal_unit_type: 34,
                                                    nalus: vec![vec![68, 1, 193, 115, 192, 137]]
                                                },
                                            ],
                                        },
                                        lhvc: None,
                                        btrt: None,
                                        colr: None,
                                        pasp: None,
                                        taic: None,
                                        fiel: None,
                                        ccst: None,
                                    }
                                    .into()],
                                },
                                stts: Stts {
                                    entries: vec![SttsEntry {
                                        sample_count: 30,
                                        sample_delta: 1000
                                    }]
                                },
                                stsc: Stsc {
                                    entries: vec![StscEntry {
                                        first_chunk: 1,
                                        samples_per_chunk: 30,
                                        sample_description_index: 1
                                    }]
                                },
                                stsz: Stsz {
                                    samples: StszSamples::Different {
                                        sizes: vec![
                                            44, 41, 27, 74, 39, 44, 41, 27, 74, 39, 44, 41, 27, 74,
                                            39, 44, 41, 27, 74, 39, 44, 41, 27, 74, 39, 44, 41, 27,
                                            74, 39
                                        ]
                                    }
                                },
                                stco: Some(Stco {
                                    entries: vec![1686]
                                }),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    },
                    ..Default::default()
                },
                Trak {
                    tkhd: Tkhd {
                        creation_time: 3732765738,
                        modification_time: 3732765738,
                        track_id: 2,
                        duration: 600,
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
                        width: 0.into(),
                        height: 0.into(),
                    },
                    mdia: Mdia {
                        mdhd: Mdhd {
                            creation_time: 3732765738,
                            modification_time: 3732765738,
                            timescale: 30000,
                            duration: 30000,
                            language: "und".into(),
                        },
                        hdlr: Hdlr {
                            handler: FourCC::new(b"meta"),
                            name: "meta".into(),
                        },
                        minf: Minf {
                            nmhd: Some(Nmhd {}),
                            dinf: Dinf {
                                dref: Dref {
                                    urls: vec![Url {
                                        location: "".into()
                                    }]
                                }
                            },
                            stbl: Stbl {
                                stsd: Stsd {
                                    codecs: vec![Mebx {
                                        metadata: MetaData {
                                            data_reference_index: 1
                                        },
                                        keys: vec![
                                            key(b"redd", "Config RED", None),
                                            key(b"blue", "Config BLUE", None),
                                            key(b"ylow", "Config YELLOW", None),
                                            key(b"whte", "Config WHITE", None),
                                            key(b"blck", "Config BLACK", None),
                                            key(b"labl", "Config lable ENGLISH", Some("en-US")),
                                            key(b"labl", "Config lable GERMAN", Some("de-DE")),
                                        ],
                                        btrt: None,
                                    }
                                    .into()],
                                },
                                stts: Stts {
                                    entries: vec![SttsEntry {
                                        sample_count: 30,
                                        sample_delta: 1000
                                    }]
                                },
                                stsc: Stsc {
                                    entries: vec![StscEntry {
                                        first_chunk: 1,
                                        samples_per_chunk: 30,
                                        sample_description_index: 1
                                    }]
                                },
                                stsz: Stsz {
                                    samples: StszSamples::Different {
                                        sizes: vec![
                                            72, 48, 24, 72, 72, 72, 48, 24, 72, 72, 72, 48, 24, 72,
                                            72, 72, 48, 24, 72, 72, 72, 48, 24, 72, 72, 72, 48, 24,
                                            72, 72
                                        ]
                                    }
                                },
                                stco: Some(Stco {
                                    entries: vec![3036]
                                }),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    },
                    tref: Some(Tref {
                        track_reference_type_boxes: vec![TrackReferenceTypeBox {
                            reference_type: FourCC::new(b"cdsc"),
                            track_ids: vec![1]
                        }]
                    }),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    );

    let mdat = Mdat::decode(buf).expect("failed to decode mdat");
    assert_eq!(mdat.data.len(), 3078);

    // The file ends right after mdat.
    assert!(!buf.has_remaining());

    // Round-trip: re-encode and decode everything back.
    let mut reencoded = Vec::new();
    ftyp.encode(&mut reencoded).expect("failed to encode ftyp");
    moov.encode(&mut reencoded).expect("failed to encode moov");
    mdat.encode(&mut reencoded).expect("failed to encode mdat");

    let reencoded_buf = &mut reencoded.as_slice();
    let reencoded_ftyp = Ftyp::decode(reencoded_buf).expect("failed to re-decode ftyp");
    let reencoded_moov = Moov::decode(reencoded_buf).expect("failed to re-decode moov");
    let reencoded_mdat = Mdat::decode(reencoded_buf).expect("failed to re-decode mdat");
    assert_eq!(reencoded_ftyp, ftyp);
    assert_eq!(reencoded_moov, moov);
    assert_eq!(reencoded_mdat, mdat);
}
