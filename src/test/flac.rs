use crate::*;

#[test]
fn flac() {
    const ENCODED: &[u8] = include_bytes!("flac.mp4");

    let buf = &mut std::io::Cursor::new(&ENCODED);

    let ftyp = Ftyp::decode(buf).expect("failed to decode ftyp");

    assert_eq!(
        ftyp,
        Ftyp {
            major_brand: b"iso6".into(),
            minor_version: 0,
            compatible_brands: vec![b"iso6".into(),],
        }
    );

    let moov = Moov::decode(buf).expect("failed to decode moov");
    assert_eq!(
        moov,
        Moov {
            mvhd: Mvhd {
                creation_time: 3840517353,
                modification_time: 3840517353,
                timescale: 44100,
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
                    creation_time: 3840517353,
                    modification_time: 3840517353,
                    track_id: 1,
                    duration: 0,
                    layer: 0,
                    alternate_group: 0,
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
                        w: 1073741824
                    },
                    width: 0.into(),
                    height: 0.into(),
                },
                mdia: Mdia {
                    mdhd: Mdhd {
                        creation_time: 3840517353,
                        modification_time: 3840517353,
                        timescale: 44100,
                        duration: 0,
                        language: "und".into(),
                    },
                    hdlr: Hdlr {
                        handler: b"soun".into(),
                        name: "SoundHandler".into(),
                    },
                    minf: Minf {
                        smhd: Some(Smhd { balance: 0.into() }),
                        dinf: Dinf {
                            dref: Dref {
                                urls: vec![Url {
                                    location: "".into()
                                }]
                            }
                        },
                        stbl: Stbl {
                            stsd: Stsd {
                                codecs: vec![Flac {
                                    audio: Audio {
                                        data_reference_index: 1,
                                        channel_count: 1,
                                        sample_size: 8,
                                        sample_rate: 44100.into(),
                                    },
                                    dfla: Dfla {
                                        blocks: vec![
                                            FlacMetadataBlock::StreamInfo {
                                                minimum_block_size: 4608,
                                                maximum_block_size: 4608,
                                                minimum_frame_size: 0u32
                                                    .try_into()
                                                    .expect("should fit in u24"),
                                                maximum_frame_size: 0u32
                                                    .try_into()
                                                    .expect("should fit in u24"),
                                                sample_rate: 44100,
                                                num_channels_minus_one: 0,
                                                bits_per_sample_minus_one: 7,
                                                number_of_interchannel_samples: 0,
                                                md5_checksum: vec![
                                                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                                                ]
                                            },
                                            FlacMetadataBlock::VorbisComment {
                                                vendor_string: "reference libFLAC 1.4.3 20230623"
                                                    .into(),
                                                comments: vec!["DESCRIPTION=audiotest wave".into()]
                                            },
                                        ],
                                    },
                                }
                                .into(),]
                            },
                            stts: Stts { entries: vec![] },
                            ctts: None,
                            stss: None,
                            stsc: Stsc { entries: vec![] },
                            stsz: Stsz::default(),
                            stco: Some(Stco { entries: [].into() }),
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
            ..Default::default()
        }
    );

    // Make sure the FLAC atom encodes/decodes to the exact same content.
    let flac = &moov.trak[0].mdia.minf.stbl.stsd.codecs[0];
    flac.assert_encode_decode();

    let mut buf = Vec::new();
    ftyp.encode(&mut buf).expect("failed to encode ftyp");
    moov.encode(&mut buf).expect("failed to encode moov");
}
