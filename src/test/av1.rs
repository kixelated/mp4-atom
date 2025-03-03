use crate::*;

#[test]
fn av1() {
    const ENCODED: &[u8] = &[
        0x00, 0x00, 0x00, 0x20, 0x66, 0x74, 0x79, 0x70, 0x69, 0x73, 0x6F, 0x36, 0x00, 0x00, 0x02,
        0x00, 0x69, 0x73, 0x6F, 0x36, 0x63, 0x6D, 0x66, 0x63, 0x61, 0x76, 0x30, 0x31, 0x6D, 0x70,
        0x34, 0x31, 0x00, 0x00, 0x02, 0xF8, 0x6D, 0x6F, 0x6F, 0x76, 0x00, 0x00, 0x00, 0x6C, 0x6D,
        0x76, 0x68, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x03, 0xE8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00,
        0x01, 0xFB, 0x74, 0x72, 0x61, 0x6B, 0x00, 0x00, 0x00, 0x5C, 0x74, 0x6B, 0x68, 0x64, 0x00,
        0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00,
        0x07, 0x80, 0x00, 0x00, 0x04, 0x38, 0x00, 0x00, 0x00, 0x00, 0x01, 0x97, 0x6D, 0x64, 0x69,
        0x61, 0x00, 0x00, 0x00, 0x20, 0x6D, 0x64, 0x68, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x61, 0xA8, 0x00, 0x00, 0x00, 0x00, 0x55,
        0xC4, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50, 0x68, 0x64, 0x6C, 0x72, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x76, 0x69, 0x64, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x6F, 0x62, 0x75, 0x40, 0x47, 0x50, 0x41, 0x43, 0x32, 0x2E,
        0x31, 0x2D, 0x44, 0x45, 0x56, 0x2D, 0x72, 0x65, 0x76, 0x31, 0x39, 0x39, 0x2D, 0x67, 0x38,
        0x65, 0x32, 0x39, 0x66, 0x36, 0x65, 0x38, 0x62, 0x2D, 0x67, 0x69, 0x74, 0x68, 0x75, 0x62,
        0x5F, 0x6D, 0x61, 0x73, 0x74, 0x65, 0x72, 0x00, 0x00, 0x00, 0x01, 0x1F, 0x6D, 0x69, 0x6E,
        0x66, 0x00, 0x00, 0x00, 0x14, 0x76, 0x6D, 0x68, 0x64, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x24, 0x64, 0x69, 0x6E, 0x66, 0x00,
        0x00, 0x00, 0x1C, 0x64, 0x72, 0x65, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x0C, 0x75, 0x72, 0x6C, 0x20, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0xDF, 0x73, 0x74, 0x62, 0x6C, 0x00, 0x00, 0x00, 0x93, 0x73, 0x74, 0x73, 0x64, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x83, 0x61, 0x76, 0x30, 0x31, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x80, 0x04, 0x38, 0x00, 0x48, 0x00,
        0x00, 0x00, 0x48, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18,
        0xFF, 0xFF, 0x00, 0x00, 0x00, 0x19, 0x61, 0x76, 0x31, 0x43, 0x81, 0x09, 0x4C, 0x00, 0x0A,
        0x0B, 0x00, 0x00, 0x00, 0x4A, 0xAB, 0xBF, 0xC3, 0x77, 0xFF, 0xE7, 0x01, 0x00, 0x00, 0x00,
        0x14, 0x62, 0x74, 0x72, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0D, 0xF4, 0x40, 0x00, 0x0D,
        0xF4, 0x40, 0x00, 0x00, 0x00, 0x10, 0x73, 0x74, 0x74, 0x73, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x73, 0x74, 0x73, 0x63, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x14, 0x73, 0x74, 0x73, 0x7A, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x73, 0x74,
        0x63, 0x6F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x28, 0x6D,
        0x76, 0x65, 0x78, 0x00, 0x00, 0x00, 0x20, 0x74, 0x72, 0x65, 0x78, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x61, 0x75, 0x64, 0x74, 0x61, 0x00, 0x00,
        0x00, 0x59, 0x6D, 0x65, 0x74, 0x61, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0x68,
        0x64, 0x6C, 0x72, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x6D, 0x64, 0x69, 0x72,
        0x61, 0x70, 0x70, 0x6C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x2C, 0x69, 0x6C, 0x73, 0x74, 0x00, 0x00, 0x00, 0x24, 0xA9, 0x74, 0x6F, 0x6F, 0x00,
        0x00, 0x00, 0x1C, 0x64, 0x61, 0x74, 0x61, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x4C, 0x61, 0x76, 0x66, 0x36, 0x31, 0x2E, 0x37, 0x2E, 0x31, 0x30, 0x30, 0x00, 0x00, 0x00,
        0x6C, 0x6D, 0x6F, 0x6F, 0x66, 0x00, 0x00, 0x00, 0x10, 0x6D, 0x66, 0x68, 0x64, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x54, 0x74, 0x72, 0x61, 0x66, 0x00,
        0x00, 0x00, 0x20, 0x74, 0x66, 0x68, 0x64, 0x00, 0x02, 0x00, 0x3A, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x03, 0xE8, 0x00, 0x00, 0x00, 0xFC, 0x01, 0x01, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x14, 0x74, 0x66, 0x64, 0x74, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x74, 0x72, 0x75, 0x6E, 0x01,
        0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x74, 0x02, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x01, 0x04, 0x6D, 0x64, 0x61, 0x74, 0x0A, 0x0B, 0x00, 0x00, 0x00, 0x4A, 0xAB,
        0xBF, 0xC3, 0x77, 0xFF, 0xE7, 0x01, 0x32, 0xEC, 0x01, 0x10, 0x00, 0xE0, 0x10, 0x00, 0x00,
        0x80, 0x00, 0x30, 0x00, 0x40, 0x00, 0x10, 0x30, 0xC0, 0x00, 0x36, 0xF4, 0x1F, 0x4E, 0xCF,
        0xD3, 0x33, 0x40, 0xCA, 0xF8, 0xDF, 0x0C, 0xA3, 0x4E, 0x1A, 0x2D, 0x0B, 0x96, 0x0D, 0x5A,
        0x19, 0xB0, 0x7B, 0xFF, 0xEC, 0x45, 0x84, 0xF0, 0x97, 0xB2, 0xF9, 0x1D, 0xA6, 0x96, 0x54,
        0x14, 0xFA, 0x08, 0x0C, 0x95, 0x3F, 0x77, 0xA8, 0x09, 0xB7, 0xE1, 0x04, 0x27, 0x72, 0x33,
        0xA5, 0x28, 0x12, 0x8C, 0xF9, 0x6D, 0x36, 0xF4, 0x1F, 0x4E, 0xCF, 0xD3, 0x33, 0x40, 0xCA,
        0xF8, 0xDF, 0x0C, 0xA3, 0x4E, 0x1A, 0x2D, 0x0B, 0x96, 0x0D, 0x5A, 0x19, 0xB0, 0x7B, 0xFF,
        0xEC, 0x45, 0x84, 0xF0, 0x97, 0xB2, 0xF9, 0x1D, 0xA6, 0x96, 0x54, 0x14, 0xFA, 0x08, 0x0C,
        0x95, 0x3F, 0x77, 0xA8, 0x09, 0xB7, 0xE1, 0x04, 0x27, 0x72, 0x33, 0xA5, 0x28, 0x12, 0x8C,
        0xF9, 0x6D, 0x36, 0xF4, 0x1F, 0x4E, 0xCF, 0xD3, 0x33, 0x40, 0xCA, 0xF8, 0xDF, 0x0C, 0xA3,
        0x4E, 0x1A, 0x2D, 0x0B, 0x96, 0x0D, 0x5A, 0x19, 0xB0, 0x7B, 0xFF, 0xEC, 0x45, 0x84, 0xF0,
        0x97, 0xB2, 0xF9, 0x1D, 0xA6, 0x96, 0x54, 0x14, 0xFA, 0x08, 0x0C, 0x95, 0x3F, 0x77, 0xA8,
        0x09, 0xB7, 0xE1, 0x04, 0x27, 0x72, 0x33, 0xA5, 0x28, 0x12, 0x8C, 0xF9, 0x6D, 0xF4, 0x1F,
        0x4E, 0xCF, 0xD3, 0x33, 0x40, 0xCA, 0xF8, 0xDF, 0x0C, 0xA3, 0x4E, 0x1A, 0x2D, 0x0B, 0x96,
        0x0D, 0x5A, 0x19, 0xB0, 0x7B, 0xFF, 0xEC, 0x45, 0x83, 0x8F, 0xDE, 0x1C, 0x95, 0x83, 0xFE,
        0xFF, 0xBA, 0x11, 0x74, 0x91, 0xF2, 0xCE, 0x48, 0xE4, 0xA5, 0xEE, 0x59, 0x8D, 0x17, 0x46,
        0x00, 0x09, 0x4E, 0x67, 0x60, 0x00, 0x00, 0x00, 0x68, 0x6D, 0x6F, 0x6F, 0x66, 0x00, 0x00,
        0x00, 0x10, 0x6D, 0x66, 0x68, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00,
        0x00, 0x00, 0x50, 0x74, 0x72, 0x61, 0x66, 0x00, 0x00, 0x00, 0x20, 0x74, 0x66, 0x68, 0x64,
        0x00, 0x02, 0x00, 0x3A, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x03,
        0xE8, 0x00, 0x00, 0x01, 0xC5, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x14, 0x74, 0x66,
        0x64, 0x74, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xE8, 0x00,
        0x00, 0x00, 0x14, 0x74, 0x72, 0x75, 0x6E, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x70,
    ];

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

    //   left: Moov { mvhd: Mvhd { creation_time: 0, modification_time: 0, timescale: 1000, duration: 0, rate: 1, volume: 1, matrix: Matrix { a: 65536, b: 0, u: 0, c: 0, d: 65536, v: 0, x: 0, y: 0, w: 1073741824 }, next_track_id: 2 }, meta: None, mvex: Some(Mvex { mehd: None, trex: [Trex { track_id: 1, default_sample_description_index: 1, default_sample_duration: 0, default_sample_size: 0, default_sample_flags: 0 }] }), trak: [Trak { tkhd: Tkhd { creation_time: 0, modification_time: 0, track_id: 1, duration: 0, layer: 0, alternate_group: 0, enabled: true, volume: 0, matrix: Matrix { a: 65536, b: 0, u: 0, c: 0, d: 65536, v: 0, x: 0, y: 0, w: 1073741824 }, width: 1920, height: 1080 }, edts: None, meta: None, mdia: Mdia { mdhd: Mdhd { creation_time: 0, modification_time: 0, timescale: 25000, duration: 0, language: "und" }, hdlr: Hdlr { handler: vide, name: "obu@GPAC2.1-DEV-rev199-g8e29f6e8b-github_master" }, minf: Minf { vmhd: Some(Vmhd { graphics_mode: 0, op_color: RgbColor { red: 0, green: 0, blue: 0 } }), smhd: None, dinf: Dinf { dref: Dref { urls: [Url { location: "" }] } }, stbl: Stbl { stsd: Stsd { avc1: None, hev1: None, vp09: None, mp4a: None, tx3g: None, av01: Some(Av01 { visual: Visual { data_reference_index: 1, width: 1920, height: 1080, horizresolution: 72, vertresolution: 72, frame_count: 1, compressor: Compressor(""), depth: 24 }, av1c: Av1c { seq_profile: 0, seq_level_idx_0: 9, seq_tier_0: false, high_bitdepth: true, twelve_bit: false, monochrome: false, chroma_subsampling_x: true, chroma_subsampling_y: true, chroma_sample_position: 0, initial_presentation_delay: None, config_obus: [10, 11, 0, 0, 0, 74, 171, 191, 195, 119, 255, 231, 1] } }) }, stts: Stts { entries: [] }, ctts: None, stss: None, stsc: Stsc { entries: [] }, stsz: Stsz { samples: Different { sizes: [] } }, stco: Some(Stco { entries: [] }), co64: None } } } }], udta: Some(Udta { meta: Some(Mdir { ilst: Some(Ilst { name: None, year: None, covr: None, desc: None }) }), skip: None }) }

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
            meta: None,
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
                        smhd: None,
                        dinf: Dinf {
                            dref: Dref {
                                urls: vec![Url {
                                    location: "".into()
                                }]
                            }
                        },
                        stbl: Stbl {
                            stsd: Stsd {
                                avc1: None,
                                hev1: None,
                                vp09: None,
                                mp4a: None,
                                tx3g: None,
                                av01: Some(Av01 {
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
                                    }
                                })
                            },
                            stts: Stts::default(),
                            ctts: None,
                            stss: None,
                            stsc: Stsc::default(),
                            stsz: Stsz::default(),
                            stco: Some(Stco::default()),
                            co64: None
                        }
                    }
                }
            }],
            udta: Some(Udta {
                meta: Some(Meta::Mdir {
                    ilst: Some(Ilst {
                        name: None,
                        year: None,
                        covr: None,
                        desc: None
                    })
                }),
                skip: None
            })
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
                trun: Some(Trun {
                    data_offset: Some(116),
                    entries: vec![TrunEntry {
                        duration: None,
                        size: None,
                        flags: Some(33554432),
                        cts: None
                    }]
                })
            }]
        }
    );

    let mut buf = Vec::new();
    ftyp.encode(&mut buf).expect("failed to encode ftyp");
    moov.encode(&mut buf).expect("failed to encode moov");
    moof.encode(&mut buf).expect("failed to encode moof");

    // assert_eq!(buf, ENCODED);
}
