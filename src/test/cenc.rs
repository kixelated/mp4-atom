use crate::*;

// ClearKey's system ID, and the key ID the fixture is encrypted under.
const CLEARKEY: [u8; 16] = [
    0x10, 0x77, 0xef, 0xec, 0xc0, 0xb2, 0x4d, 0x02, 0xac, 0xe3, 0x3c, 0x1e, 0x52, 0xe2, 0xfb, 0x4b,
];
const KID: [u8; 16] = [
    0xa1, 0xb2, 0xc3, 0xd4, 0xe5, 0xf6, 0x07, 0x18, 0x29, 0x3a, 0x4b, 0x5c, 0x6d, 0x7e, 0x8f, 0x90,
];

/// The init segment of a ClearKey `cenc` file: ffmpeg's `-encryption_scheme cenc-aes-ctr`
/// on H.264 and AAC, with a version 1 ClearKey `pssh` appended to the moov.
#[test]
fn cenc_init() {
    const ENCODED: &[u8] = include_bytes!("cenc_init.mp4");
    let buf = &mut std::io::Cursor::new(&ENCODED);

    Ftyp::decode(buf).expect("failed to decode ftyp");
    let moov = Moov::decode(buf).expect("failed to decode moov");

    let sinf = |scheme: &[u8; 4], format: &[u8; 4]| Sinf {
        frma: Frma {
            data_format: format.into(),
        },
        schm: Some(Schm {
            scheme_type: scheme.into(),
            scheme_version: 0x0001_0000,
            scheme_uri: None,
        }),
        schi: Some(Schi {
            tenc: Some(Tenc {
                default_pattern: None,
                default_is_protected: true,
                default_per_sample_iv_size: 8,
                default_kid: KID,
                default_constant_iv: None,
            }),
        }),
    };

    let video = &moov.trak[0].mdia.minf.stbl.stsd.codecs[0];
    let Codec::Encv(encv) = video else {
        panic!("expected encv, got {video:?}");
    };
    assert!(matches!(*encv.codec, Codec::Avc1(_)));
    assert_eq!(encv.sinf, sinf(b"cenc", b"avc1"));
    assert_eq!(video.visual(), encv.codec.visual());
    assert_eq!(
        video.visual().map(|v| (v.width, v.height)),
        Some((320, 180))
    );

    let audio = &moov.trak[1].mdia.minf.stbl.stsd.codecs[0];
    let Codec::Enca(enca) = audio else {
        panic!("expected enca, got {audio:?}");
    };
    assert!(matches!(*enca.codec, Codec::Mp4a(_)));
    assert_eq!(enca.sinf, sinf(b"cenc", b"mp4a"));
    assert_eq!(audio.audio(), enca.codec.audio());

    assert_eq!(
        moov.pssh,
        vec![Pssh {
            system_id: CLEARKEY,
            kid: Some(vec![KID]),
            data: vec![],
        }]
    );

    // The child order is this crate's, so the bytes may differ from ffmpeg's,
    // but the boxes survive and a second round is stable.
    let mut encoded = Vec::new();
    moov.encode(&mut encoded).unwrap();
    let decoded = Moov::decode(&mut encoded.as_slice()).unwrap();
    assert_eq!(decoded, moov);
    let mut again = Vec::new();
    decoded.encode(&mut again).unwrap();
    assert_eq!(again, encoded);
}

/// The init segments Shaka Packager v3.9.3 writes for `cbcs` with a ClearKey `pssh`
/// (`--protection_scheme cbcs --protection_systems CommonSystem`), one per stream:
/// H.264 with a 1:9 pattern, and AAC fully encrypted (pattern 0:0). Both use a constant IV.
#[test]
fn cbcs_shaka_inits() {
    // (init, pattern, constant IV)
    type Case = (&'static [u8], Option<(u8, u8)>, [u8; 16]);
    let cases: [Case; 2] = [
        (
            include_bytes!("cbcs_video_init.mp4"),
            Some((1, 9)),
            [
                0x0d, 0xbc, 0x16, 0x34, 0xbc, 0xa7, 0x3f, 0xfb, 0x4c, 0x95, 0x94, 0x9a, 0x4f, 0x2b,
                0x01, 0xcb,
            ],
        ),
        (
            include_bytes!("cbcs_audio_init.mp4"),
            Some((0, 0)),
            [
                0xc9, 0x9e, 0x41, 0x28, 0x97, 0xf5, 0x2f, 0x5b, 0xea, 0x26, 0xee, 0x35, 0xf2, 0xa0,
                0x61, 0xc6,
            ],
        ),
    ];

    for (encoded, pattern, iv) in cases {
        let buf = &mut std::io::Cursor::new(encoded);
        Ftyp::decode(buf).expect("failed to decode ftyp");
        let moov = Moov::decode(buf).expect("failed to decode moov");

        let codec = &moov.trak[0].mdia.minf.stbl.stsd.codecs[0];
        let sinf = match codec {
            Codec::Encv(encv) => {
                assert!(matches!(*encv.codec, Codec::Avc1(_)));
                assert_eq!(
                    codec.visual().map(|v| (v.width, v.height)),
                    Some((320, 180))
                );
                &encv.sinf
            }
            Codec::Enca(enca) => {
                assert!(matches!(*enca.codec, Codec::Mp4a(_)));
                assert_eq!(codec.audio().map(|a| a.channel_count), Some(2));
                &enca.sinf
            }
            other => panic!("expected a protected entry, got {other:?}"),
        };

        assert_eq!(
            sinf.schm,
            Some(Schm {
                scheme_type: b"cbcs".into(),
                scheme_version: 0x0001_0000,
                scheme_uri: None,
            })
        );
        assert_eq!(
            sinf.schi.as_ref().and_then(|schi| schi.tenc.clone()),
            Some(Tenc {
                default_pattern: pattern,
                default_is_protected: true,
                default_per_sample_iv_size: 0,
                default_kid: KID,
                default_constant_iv: Some(iv.to_vec()),
            })
        );
        assert_eq!(
            moov.pssh,
            vec![Pssh {
                system_id: CLEARKEY,
                kid: Some(vec![KID]),
                data: vec![],
            }]
        );

        let mut reencoded = Vec::new();
        moov.encode(&mut reencoded).unwrap();
        let decoded = Moov::decode(&mut reencoded.as_slice()).unwrap();
        assert_eq!(decoded, moov);
        let mut again = Vec::new();
        decoded.encode(&mut again).unwrap();
        assert_eq!(again, reencoded);
    }
}

/// A `cbcs` entry: `tenc` version 1 with a 1:9 pattern and a constant IV, no per-sample IV.
#[test]
fn cbcs_tenc_round_trips() {
    #[rustfmt::skip]
    const ENCODED: &[u8] = &[
        0x00, 0x00, 0x00, 0x31, b't', b'e', b'n', b'c', 0x01, 0x00, 0x00, 0x00,
        0x00, // reserved
        0x19, // crypt 1, skip 9
        0x01, // protected
        0x00, // no per-sample IV
        0xa1, 0xb2, 0xc3, 0xd4, 0xe5, 0xf6, 0x07, 0x18, 0x29, 0x3a, 0x4b, 0x5c, 0x6d, 0x7e, 0x8f, 0x90,
        0x10, // constant IV size
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    ];

    let tenc = Tenc::decode(&mut &ENCODED[..]).unwrap();
    assert_eq!(
        tenc,
        Tenc {
            default_pattern: Some((1, 9)),
            default_is_protected: true,
            default_per_sample_iv_size: 0,
            default_kid: KID,
            default_constant_iv: Some((0..16).collect()),
        }
    );

    let mut encoded = Vec::new();
    tenc.encode(&mut encoded).unwrap();
    assert_eq!(encoded, ENCODED);
}

#[test]
fn tenc_refuses_a_constant_iv_it_cannot_signal() {
    let tenc = Tenc {
        default_is_protected: true,
        default_per_sample_iv_size: 8,
        default_constant_iv: Some(vec![0; 8]),
        ..Default::default()
    };
    assert!(matches!(
        tenc.encode(&mut Vec::new()),
        Err(Error::InvalidCombination(_))
    ));

    let tenc = Tenc {
        default_is_protected: true,
        default_per_sample_iv_size: 0,
        default_constant_iv: None,
        ..Default::default()
    };
    assert!(matches!(
        tenc.encode(&mut Vec::new()),
        Err(Error::InvalidCombination(_))
    ));
}

#[test]
fn pssh_versions_round_trip() {
    for pssh in [
        Pssh {
            system_id: CLEARKEY,
            kid: None,
            data: vec![1, 2, 3],
        },
        Pssh {
            system_id: CLEARKEY,
            kid: Some(vec![]),
            data: vec![],
        },
        Pssh {
            system_id: CLEARKEY,
            kid: Some(vec![KID, [0; 16]]),
            data: vec![4, 5],
        },
    ] {
        let mut encoded = Vec::new();
        pssh.encode(&mut encoded).unwrap();
        assert_eq!(encoded[8], u8::from(pssh.kid.is_some()), "version");
        assert_eq!(Pssh::decode(&mut encoded.as_slice()).unwrap(), pssh);
    }
}

#[test]
fn pssh_refuses_a_kid_count_past_the_box() {
    #[rustfmt::skip]
    let encoded: &[u8] = &[
        0x00, 0x00, 0x00, 0x24, b'p', b's', b's', b'h', 0x01, 0x00, 0x00, 0x00,
        0x10, 0x77, 0xef, 0xec, 0xc0, 0xb2, 0x4d, 0x02, 0xac, 0xe3, 0x3c, 0x1e, 0x52, 0xe2, 0xfb, 0x4b,
        0xff, 0xff, 0xff, 0xff, // KID count
        0x00, 0x00, 0x00, 0x00,
    ];
    assert!(Pssh::decode(&mut &encoded[..]).is_err());
}

#[test]
fn schm_with_uri_round_trips() {
    let schm = Schm {
        scheme_type: b"cbcs".into(),
        scheme_version: 0x0001_0000,
        scheme_uri: Some("https://example.com/scheme".into()),
    };
    let mut encoded = Vec::new();
    schm.encode(&mut encoded).unwrap();
    assert_eq!(encoded[11], 1, "scheme_uri_present flag");
    assert_eq!(Schm::decode(&mut encoded.as_slice()).unwrap(), schm);
}

fn avc1() -> Avc1 {
    Avc1 {
        visual: Visual {
            data_reference_index: 1,
            width: 320,
            height: 240,
            ..Default::default()
        },
        avcc: Avcc {
            configuration_version: 1,
            avc_profile_indication: 66,
            avc_level_indication: 30,
            length_size: 4,
            ..Default::default()
        },
        btrt: None,
        colr: Vec::new(),
        pasp: None,
        taic: None,
        fiel: None,
        spatial: None,
    }
}

fn cenc_sinf(format: &[u8; 4]) -> Sinf {
    Sinf {
        frma: Frma {
            data_format: format.into(),
        },
        schm: Some(Schm {
            scheme_type: b"cenc".into(),
            scheme_version: 0x0001_0000,
            scheme_uri: None,
        }),
        schi: Some(Schi {
            tenc: Some(Tenc {
                default_is_protected: true,
                default_per_sample_iv_size: 8,
                default_kid: KID,
                ..Default::default()
            }),
        }),
    }
}

/// The protected entry is the original's bytes, renamed, with `sinf` appended.
#[test]
fn encv_wraps_the_original_entry() {
    let encv = Encv {
        codec: Box::new(avc1().into()),
        sinf: cenc_sinf(b"avc1"),
    };
    let stsd = Stsd {
        codecs: vec![encv.clone().into()],
    };

    let mut encoded = Vec::new();
    stsd.encode(&mut encoded).unwrap();
    assert_eq!(Stsd::decode(&mut encoded.as_slice()).unwrap(), stsd);

    let mut original = Vec::new();
    avc1().encode(&mut original).unwrap();
    let mut sinf = Vec::new();
    encv.sinf.encode(&mut sinf).unwrap();
    let mut protected = Vec::new();
    encv.encode(&mut protected).unwrap();
    assert_eq!(&protected[4..8], b"encv");
    assert_eq!(&protected[8..original.len()], &original[8..]);
    assert_eq!(&protected[original.len()..], &sinf[..]);
}

/// `sinf` may come anywhere among the entry's children.
#[test]
fn encv_finds_sinf_between_children() {
    let mut original = avc1();
    original.btrt = Some(Btrt {
        buffer_size_db: 1,
        max_bitrate: 2,
        avg_bitrate: 3,
    });
    let sinf = cenc_sinf(b"avc1");

    // fields, avcC, sinf, btrt
    let mut body = Vec::new();
    original.visual.encode(&mut body).unwrap();
    original.avcc.encode(&mut body).unwrap();
    sinf.encode(&mut body).unwrap();
    original.btrt.encode(&mut body).unwrap();
    let mut encoded = Vec::new();
    encoded
        .encode_atom(Encv::KIND, |buf| {
            buf.append_slice(&body);
            Ok(())
        })
        .unwrap();

    let encv = Encv::decode(&mut encoded.as_slice()).unwrap();
    assert_eq!(*encv.codec, Codec::Avc1(original));
    assert_eq!(encv.sinf, sinf);
}

#[test]
fn enca_round_trips() {
    let opus = opus();
    let enca = Enca {
        codec: Box::new(opus.into()),
        sinf: cenc_sinf(b"Opus"),
    };
    let mut encoded = Vec::new();
    enca.encode(&mut encoded).unwrap();
    assert_eq!(Enca::decode(&mut encoded.as_slice()).unwrap(), enca);
}

#[test]
fn encv_refuses_a_frma_that_names_another_type() {
    let encv = Encv {
        codec: Box::new(avc1().into()),
        sinf: cenc_sinf(b"hvc1"),
    };
    assert!(matches!(
        encv.encode(&mut Vec::new()),
        Err(Error::InvalidCombination(_))
    ));
}

#[test]
fn encv_refuses_a_missing_sinf() {
    let mut encoded = Vec::new();
    avc1().encode(&mut encoded).unwrap();
    encoded[4..8].copy_from_slice(b"encv");
    assert!(matches!(
        Encv::decode(&mut encoded.as_slice()),
        Err(Error::MissingBox(kind)) if kind == Sinf::KIND
    ));
}

#[test]
fn encv_refuses_to_wrap_itself() {
    let mut encoded = Vec::new();
    avc1().encode(&mut encoded).unwrap();
    encoded[4..8].copy_from_slice(b"encv");
    let mut sinf = Vec::new();
    cenc_sinf(b"encv").encode(&mut sinf).unwrap();
    encoded.extend_from_slice(&sinf);
    let size = (encoded.len() as u32).to_be_bytes();
    encoded[..4].copy_from_slice(&size);
    assert!(matches!(
        Encv::decode(&mut encoded.as_slice()),
        Err(Error::InvalidCombination(_))
    ));
}

/// Key rotation puts `pssh` in the `moof`.
#[test]
fn moof_keeps_pssh() {
    let moof = Moof {
        mfhd: Mfhd { sequence_number: 1 },
        pssh: vec![Pssh {
            system_id: CLEARKEY,
            kid: Some(vec![KID]),
            data: vec![],
        }],
        traf: vec![],
    };
    let mut encoded = Vec::new();
    moof.encode(&mut encoded).unwrap();
    assert_eq!(Moof::decode(&mut encoded.as_slice()).unwrap(), moof);
}

fn opus() -> Opus {
    Opus {
        audio: Audio {
            data_reference_index: 1,
            channel_count: 2,
            sample_size: 16,
            sample_rate: 48000.into(),
        },
        dops: Dops {
            output_channel_count: 2,
            pre_skip: 312,
            input_sample_rate: 48000,
            output_gain: 0,
            channel_mapping: None,
        },
        btrt: None,
    }
}

/// `encv` wraps a visual entry and `enca` an audio one, or the result would not decode.
#[test]
fn protected_entries_refuse_the_wrong_media() {
    let encv = Encv {
        codec: Box::new(opus().into()),
        sinf: cenc_sinf(b"Opus"),
    };
    assert!(matches!(
        encv.encode(&mut Vec::new()),
        Err(Error::InvalidCombination(_))
    ));

    let enca = Enca {
        codec: Box::new(avc1().into()),
        sinf: cenc_sinf(b"avc1"),
    };
    assert!(matches!(
        enca.encode(&mut Vec::new()),
        Err(Error::InvalidCombination(_))
    ));
}

#[test]
fn encv_refuses_to_encode_a_nested_entry() {
    let inner = Encv {
        codec: Box::new(avc1().into()),
        sinf: cenc_sinf(b"avc1"),
    };
    let outer = Encv {
        codec: Box::new(inner.into()),
        sinf: cenc_sinf(b"encv"),
    };
    assert!(matches!(
        outer.encode(&mut Vec::new()),
        Err(Error::InvalidCombination(_))
    ));
}

/// A `sinf` whose size runs past the entry is out of bounds, not missing.
#[test]
fn encv_reports_a_truncated_sinf() {
    let encv = Encv {
        codec: Box::new(avc1().into()),
        sinf: cenc_sinf(b"avc1"),
    };
    let mut encoded = Vec::new();
    encv.encode(&mut encoded).unwrap();

    let mut sinf = Vec::new();
    encv.sinf.encode(&mut sinf).unwrap();
    let at = encoded.len() - sinf.len();
    let size = u32::from_be_bytes(encoded[at..at + 4].try_into().unwrap()) + 20;
    encoded[at..at + 4].copy_from_slice(&size.to_be_bytes());

    assert!(matches!(
        Encv::decode(&mut encoded.as_slice()),
        Err(Error::OverDecode(kind)) if kind == Encv::KIND
    ));
}

#[test]
fn tenc_refuses_values_the_spec_does_not_allow() {
    let tenc = Tenc {
        default_is_protected: true,
        default_per_sample_iv_size: 8,
        default_kid: KID,
        ..Default::default()
    };
    let mut good = Vec::new();
    tenc.encode(&mut good).unwrap();

    // default_isProtected 2
    let mut bad = good.clone();
    bad[14] = 2;
    assert!(Tenc::decode(&mut bad.as_slice()).is_err());

    // per-sample IV size 5
    let mut bad = good.clone();
    bad[15] = 5;
    assert!(Tenc::decode(&mut bad.as_slice()).is_err());
    let odd = Tenc {
        default_per_sample_iv_size: 5,
        ..tenc.clone()
    };
    assert!(odd.encode(&mut Vec::new()).is_err());

    // constant IV of 3 bytes
    let odd = Tenc {
        default_per_sample_iv_size: 0,
        default_constant_iv: Some(vec![0; 3]),
        ..tenc
    };
    assert!(odd.encode(&mut Vec::new()).is_err());
}

#[test]
fn schm_refuses_a_uri_with_a_nul() {
    let schm = Schm {
        scheme_type: b"cenc".into(),
        scheme_version: 0x0001_0000,
        scheme_uri: Some("a\0b".into()),
    };
    assert!(schm.encode(&mut Vec::new()).is_err());
}

/// An `encv` whose `frma` names an audio entry does not decode.
#[test]
fn encv_refuses_to_decode_an_audio_entry() {
    let mut encoded = Vec::new();
    opus().encode(&mut encoded).unwrap();
    encoded[4..8].copy_from_slice(b"encv");
    let mut sinf = Vec::new();
    cenc_sinf(b"Opus").encode(&mut sinf).unwrap();
    encoded.extend_from_slice(&sinf);
    let size = (encoded.len() as u32).to_be_bytes();
    encoded[..4].copy_from_slice(&size);
    assert!(Encv::decode(&mut encoded.as_slice()).is_err());

    // The same entry as `enca` decodes.
    encoded[4..8].copy_from_slice(b"enca");
    let enca = Enca::decode(&mut encoded.as_slice()).unwrap();
    assert!(matches!(*enca.codec, Codec::Opus(_)));
}
