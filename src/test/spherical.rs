use crate::*;

fn atom(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
    [&((body.len() + 8) as u32).to_be_bytes()[..], kind, body].concat()
}

fn full(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
    atom(kind, &[&[0, 0, 0, 0], body].concat())
}

fn round_trip<T: Atom + std::fmt::Debug + PartialEq>(encoded: &[u8], expected: T) {
    assert_eq!(T::decode(&mut &encoded[..]).unwrap(), expected);
    let mut buf = Vec::new();
    expected.encode(&mut buf).unwrap();
    assert_eq!(buf, encoded);
}

#[test]
fn sv3d_equirectangular() {
    let encoded = atom(
        b"sv3d",
        &[
            full(b"svhd", b"Spherical Metadata Tool\0"),
            atom(
                b"proj",
                &[
                    // yaw -90°, pitch 10°, roll 0° (16.16)
                    full(b"prhd", &[0xff, 0xa6, 0, 0, 0, 0x0a, 0, 0, 0, 0, 0, 0]),
                    full(b"equi", &[0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4]),
                ]
                .concat(),
            ),
        ]
        .concat(),
    );

    round_trip(
        &encoded,
        Sv3d {
            svhd: Some(Svhd {
                metadata_source: "Spherical Metadata Tool".into(),
            }),
            proj: Some(Proj {
                prhd: Some(Prhd {
                    pose_yaw_degrees: -90 << 16,
                    pose_pitch_degrees: 10 << 16,
                    pose_roll_degrees: 0,
                }),
                kind: Some(ProjKind::Equi(Equi {
                    projection_bounds_top: 1,
                    projection_bounds_bottom: 2,
                    projection_bounds_left: 3,
                    projection_bounds_right: 4,
                })),
            }),
        },
    );
}

#[test]
fn sv3d_cubemap_and_mesh() {
    let cbmp = full(b"cbmp", &[0, 0, 0, 0, 0, 0, 0, 2]);
    round_trip(
        &atom(b"sv3d", &atom(b"proj", &cbmp)),
        Sv3d {
            svhd: None,
            proj: Some(Proj {
                prhd: None,
                kind: Some(ProjKind::Cbmp(Cbmp {
                    layout: 0,
                    padding: 2,
                })),
            }),
        },
    );

    let mshp = full(
        b"mshp",
        &[0, 0, 0x12, 0x34, b'r', b'a', b'w', b' ', 1, 2, 3],
    );
    round_trip(
        &atom(b"sv3d", &atom(b"proj", &mshp)),
        Sv3d {
            svhd: None,
            proj: Some(Proj {
                prhd: None,
                kind: Some(ProjKind::Mshp(Mshp {
                    crc: 0x1234,
                    encoding_four_cc: FourCC::new(b"raw "),
                    data: vec![1, 2, 3],
                })),
            }),
        },
    );
}

#[test]
fn sv3d_skips_padding() {
    let encoded = atom(
        b"sv3d",
        &atom(
            b"proj",
            &[atom(b"free", &[0; 4]), full(b"equi", &[0; 16])].concat(),
        ),
    );
    let decoded = Sv3d::decode(&mut encoded.as_slice()).unwrap();
    assert_eq!(
        decoded.proj.unwrap().kind,
        Some(ProjKind::Equi(Equi::default()))
    );
}

#[test]
fn sv3d_two_projections() {
    let encoded = atom(
        b"sv3d",
        &atom(
            b"proj",
            &[full(b"equi", &[0; 16]), full(b"cbmp", &[0; 8])].concat(),
        ),
    );
    assert!(matches!(
        Sv3d::decode(&mut encoded.as_slice()),
        Err(Error::DuplicateBox(_))
    ));
}

#[test]
fn st3d() {
    round_trip(
        &full(b"st3d", &[2]),
        St3d {
            stereo_mode: St3d::LEFT_RIGHT,
        },
    );
}

#[test]
fn vexu() {
    let encoded = atom(
        b"vexu",
        &[
            atom(b"proj", &full(b"prji", b"hequ")),
            atom(
                b"eyes",
                &[
                    full(b"stri", &[0b0011]),
                    full(b"hero", &[1]),
                    atom(b"cams", &full(b"blin", &63_000u32.to_be_bytes())),
                    atom(b"cmfy", &full(b"dadj", &(-200i32).to_be_bytes())),
                ]
                .concat(),
            ),
            atom(b"pack", &full(b"pkin", b"side")),
        ]
        .concat(),
    );

    round_trip(
        &encoded,
        Vexu {
            proj: Some(VexuProj {
                prji: Prji {
                    projection: Prji::HALF_EQUIRECTANGULAR,
                },
            }),
            eyes: Some(Eyes {
                stri: Some(Stri {
                    has_left_eye_view: true,
                    has_right_eye_view: true,
                    ..Default::default()
                }),
                hero: Some(Hero {
                    hero_eye: Hero::LEFT,
                }),
                cams: Some(Cams {
                    blin: Blin { baseline: 63_000 },
                }),
                cmfy: Some(Cmfy {
                    dadj: Dadj {
                        disparity_adjustment: -200,
                    },
                }),
            }),
            pack: Some(Pack {
                pkin: Some(Pkin {
                    packing: Pkin::SIDE_BY_SIDE,
                }),
            }),
        },
    );
}

#[test]
fn hfov() {
    round_trip(
        &atom(b"hfov", &90_000u32.to_be_bytes()),
        Hfov {
            horizontal_field_of_view: 90_000,
        },
    );
}

#[test]
fn sample_entry() {
    let hvc1 = Hvc1 {
        pasp: Some(Pasp {
            h_spacing: 1,
            v_spacing: 1,
        }),
        spatial: Spatial {
            st3d: Some(St3d {
                stereo_mode: St3d::TOP_BOTTOM,
            }),
            sv3d: Some(Sv3d {
                svhd: None,
                proj: Some(Proj {
                    prhd: Some(Prhd::default()),
                    kind: Some(ProjKind::Equi(Equi::default())),
                }),
            }),
            vexu: Some(Vexu {
                proj: Some(VexuProj {
                    prji: Prji {
                        projection: Prji::RECTILINEAR,
                    },
                }),
                ..Default::default()
            }),
            hfov: Some(Hfov {
                horizontal_field_of_view: 63_400,
            }),
        },
        ..Default::default()
    };

    let mut buf = Vec::new();
    hvc1.encode(&mut buf).unwrap();
    assert_eq!(Hvc1::decode(&mut buf.as_slice()).unwrap(), hvc1);

    // Spherical Video V2: `st3d` then `sv3d`, after the configuration box and
    // before the optional boxes.
    let position = |kind: &[u8]| buf.windows(4).position(|w| w == kind).unwrap();
    assert!(position(b"hvcC") < position(b"st3d"));
    assert!(position(b"st3d") < position(b"sv3d"));
    assert!(position(b"sv3d") < position(b"pasp"));
}
