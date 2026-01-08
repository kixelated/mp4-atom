use crate::*;

#[test]
fn vp9() {
    const ENCODED: &[u8] = include_bytes!("vp9.mp4");

    let buf = &mut std::io::Cursor::new(&ENCODED);

    let ftyp = Ftyp::decode(buf).expect("failed to decode ftyp");

    assert_eq!(
        ftyp,
        Ftyp {
            major_brand: b"isom".into(),
            minor_version: 0,
            compatible_brands: vec![
                b"iso8".into(),
                b"mp41".into(),
                b"dash".into(),
                b"vp09".into(),
                b"cmfc".into()
            ],
        }
    );

    let moov = Moov::decode(buf).expect("failed to decode moov");
    assert_eq!(
        moov,
        Moov {
            mvhd: Mvhd {
                creation_time: 3576083626,
                modification_time: 3576083626,
                timescale: 1000000,
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
            meta: Some(Meta {
                hdlr: Hdlr {
                    handler: FourCC::new(b"ID32"),
                    name: "".into()
                },
                items: vec![Any::Unknown(
                    FourCC::new(b"ID32"),
                    vec![
                        0, 0, 0, 0, 21, 199, 73, 68, 51, 4, 0, 0, 0, 0, 0, 67, 80, 82, 73, 86, 0,
                        0, 0, 57, 0, 0, 104, 116, 116, 112, 115, 58, 47, 47, 103, 105, 116, 104,
                        117, 98, 46, 99, 111, 109, 47, 103, 111, 111, 103, 108, 101, 47, 115, 104,
                        97, 107, 97, 45, 112, 97, 99, 107, 97, 103, 101, 114, 0, 53, 102, 99, 57,
                        48, 55, 54, 102, 57, 50, 45, 100, 101, 98, 117, 103
                    ]
                )]
            }),
            mvex: Some(Mvex {
                mehd: Some(Mehd {
                    fragment_duration: 2736000
                }),
                trex: vec![Trex {
                    track_id: 1,
                    default_sample_description_index: 1,
                    default_sample_duration: 33000,
                    default_sample_size: 0,
                    default_sample_flags: 0
                }]
            }),
            trak: vec![Trak {
                tkhd: Tkhd {
                    creation_time: 3576083626,
                    modification_time: 3576083626,
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
                    width: 320.into(),
                    height: 240.into()
                },
                edts: None,
                meta: None,
                mdia: Mdia {
                    mdhd: Mdhd {
                        creation_time: 3576083626,
                        modification_time: 3576083626,
                        timescale: 1000000,
                        duration: 0,
                        language: "und".to_string()
                    },
                    hdlr: Hdlr {
                        handler: b"vide".into(),
                        name: "VideoHandler".to_string()
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
                        smhd: None,
                        dinf: Dinf {
                            dref: Dref {
                                urls: vec![Url {
                                    location: "".to_string()
                                }]
                            }
                        },
                        stbl: Stbl {
                            stsd: Stsd {
                                codecs: vec![Vp09 {
                                    visual: Visual {
                                        data_reference_index: 1,
                                        width: 320,
                                        height: 240,
                                        horizresolution: 72.into(),
                                        vertresolution: 72.into(),
                                        frame_count: 1,
                                        compressor: "\nVPC Coding".into(),
                                        depth: 24
                                    },
                                    vpcc: VpcC {
                                        profile: 0,
                                        level: 10,
                                        bit_depth: 8,
                                        chroma_subsampling: 1,
                                        video_full_range_flag: false,
                                        color_primaries: 2,
                                        transfer_characteristics: 2,
                                        matrix_coefficients: 2,
                                        codec_initialization_data: vec![]
                                    },
                                    btrt: None,
                                    colr: None,
                                    pasp: None,
                                }
                                .into()],
                            },
                            stts: Stts { entries: vec![] },
                            ctts: None,
                            stss: None,
                            stsc: Stsc { entries: vec![] },
                            stsz: Stsz::default(),
                            stco: Some(Stco { entries: vec![] }),
                            co64: None,
                            sbgp: vec![],
                            sgpd: vec![],
                            subs: vec![],
                            saio: vec![],
                            saiz: vec![],
                            cslg: None,
                        }
                    }
                },
                udta: None,
            }],
            udta: None
        }
    );

    // Make sure the vp09 atom encodes/decodes to the exact same content.
    let vp09 = &moov.trak[0].mdia.minf.stbl.stsd.codecs[0];
    vp09.assert_encode_decode();

    let mut buf = Vec::new();

    ftyp.encode(&mut buf).expect("failed to encode ftyp");
    moov.encode(&mut buf).expect("failed to encode moov");

    // One day:
    // assert_eq!(buf, ENCODED);
}
