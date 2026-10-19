use crate::*;

fn atom(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
    [&((body.len() + 8) as u32).to_be_bytes()[..], kind, body].concat()
}

fn full_vf(kind: &[u8; 4], version: u8, flags: u32, body: &[u8]) -> Vec<u8> {
    let version_flags = ((version as u32) << 24 | flags).to_be_bytes();
    atom(kind, &[&version_flags[..], body].concat())
}

fn full(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
    full_vf(kind, 0, 0, body)
}

/// The `Any::Unknown` that keeps an encoded box verbatim.
fn kept(encoded: &[u8]) -> Any {
    Any::Unknown(
        FourCC::new(encoded[4..8].try_into().unwrap()),
        encoded[8..].to_vec(),
    )
}

/// The children of an encoded box, after `header` bytes, as (type, box bytes).
fn children(encoded: &[u8], header: usize) -> Vec<(String, &[u8])> {
    let mut children = Vec::new();
    let mut rest = &encoded[header..];
    while rest.len() >= 8 {
        let size = u32::from_be_bytes(rest[..4].try_into().unwrap()) as usize;
        children.push((
            String::from_utf8_lossy(&rest[4..8]).into_owned(),
            &rest[..size],
        ));
        rest = &rest[size..];
    }
    children
}

/// The types of the children of an encoded box, after `header` bytes.
fn child_kinds(encoded: &[u8], header: usize) -> Vec<String> {
    children(encoded, header)
        .into_iter()
        .map(|(kind, _)| kind)
        .collect()
}

fn round_trip<T: Atom + std::fmt::Debug + PartialEq>(encoded: &[u8], expected: T) {
    assert_eq!(T::decode(&mut &encoded[..]).unwrap(), expected);
    let mut buf = Vec::new();
    expected.encode(&mut buf).unwrap();
    assert_eq!(buf, encoded);
}

fn encode<T: Encode>(value: &T) -> Vec<u8> {
    let mut buf = Vec::new();
    value.encode(&mut buf).unwrap();
    buf
}

/// An `hvc1` sample entry (default fields, no hvcC) with `children` appended.
fn hvc1_with(children: &[u8]) -> Vec<u8> {
    let mut entry = encode(&Hvc1::default());
    entry.extend_from_slice(children);
    let size = entry.len() as u32;
    entry[..4].copy_from_slice(&size.to_be_bytes());
    entry
}

/// The 8-byte box header plus the 78-byte visual sample entry header.
const ENTRY_HEADER: usize = 86;

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
                unknown: vec![],
            }),
            unknown: vec![],
        },
    );
}

#[test]
fn sv3d_cubemap_and_mesh() {
    let cbmp = full(b"cbmp", &[0, 0, 0, 0, 0, 0, 0, 2]);
    round_trip(
        &atom(b"sv3d", &atom(b"proj", &cbmp)),
        Sv3d {
            proj: Some(Proj {
                kind: Some(ProjKind::Cbmp(Cbmp {
                    layout: 0,
                    padding: 2,
                })),
                ..Default::default()
            }),
            ..Default::default()
        },
    );

    let mshp = full(
        b"mshp",
        &[0, 0, 0x12, 0x34, b'd', b'f', b'l', b'8', 1, 2, 3],
    );
    round_trip(
        &atom(b"sv3d", &atom(b"proj", &mshp)),
        Sv3d {
            proj: Some(Proj {
                kind: Some(ProjKind::Mshp(Mshp {
                    crc: 0x1234,
                    encoding_four_cc: FourCC::new(b"dfl8"),
                    data: vec![1, 2, 3],
                })),
                ..Default::default()
            }),
            ..Default::default()
        },
    );
}

#[test]
fn sv3d_keeps_unknown_children() {
    // An unmodelled projection type, padding, and an unknown box in `sv3d`.
    let prhd = full(b"prhd", &[0; 12]);
    let abcd = full(b"abcd", &[1, 2, 3, 4]);
    let free = atom(b"free", &[0; 4]);
    let xyzw = atom(b"xyzw", &[9]);
    round_trip(
        &atom(
            b"sv3d",
            &[
                atom(b"proj", &[prhd, abcd.clone(), free.clone()].concat()),
                xyzw.clone(),
            ]
            .concat(),
        ),
        Sv3d {
            svhd: None,
            proj: Some(Proj {
                prhd: Some(Prhd::default()),
                kind: None,
                unknown: vec![kept(&abcd), kept(&free)],
            }),
            unknown: vec![kept(&xyzw)],
        },
    );
}

#[test]
fn sv3d_keeps_second_projection() {
    // "There must not be more than one subclass of a ProjectionDataBox".
    let cbmp = full(b"cbmp", &[0; 8]);
    round_trip(
        &atom(
            b"sv3d",
            &atom(b"proj", &[full(b"equi", &[0; 16]), cbmp.clone()].concat()),
        ),
        Sv3d {
            proj: Some(Proj {
                kind: Some(ProjKind::Equi(Equi::default())),
                unknown: vec![kept(&cbmp)],
                ..Default::default()
            }),
            ..Default::default()
        },
    );
}

#[test]
fn st3d() {
    round_trip(
        &full(b"st3d", &[2]),
        St3d {
            stereo_mode: St3d::LEFT_RIGHT,
        },
    );
    round_trip(
        &full(b"st3d", &[4]),
        St3d {
            stereo_mode: St3d::RIGHT_LEFT,
        },
    );
}

#[test]
fn google_boxes_ignore_trailing_bytes() {
    // "Parsers should ignore extra bytes at the end of a box." They are not
    // written back.
    assert_eq!(
        St3d::decode(&mut &full(b"st3d", &[2, 0xaa])[..]).unwrap(),
        St3d { stereo_mode: 2 }
    );
    assert_eq!(
        Svhd::decode(&mut &full(b"svhd", b"Tool\0\0\0")[..]).unwrap(),
        Svhd {
            metadata_source: "Tool".into()
        }
    );
    let prhd = [&[0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3][..], &[9, 9]].concat();
    assert_eq!(
        Prhd::decode(&mut &full(b"prhd", &prhd)[..]).unwrap(),
        Prhd {
            pose_yaw_degrees: 1,
            pose_pitch_degrees: 2,
            pose_roll_degrees: 3,
        }
    );
    let equi = [
        &[0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4][..],
        &[5; 4],
    ]
    .concat();
    assert_eq!(
        Equi::decode(&mut &full(b"equi", &equi)[..]).unwrap(),
        Equi {
            projection_bounds_top: 1,
            projection_bounds_bottom: 2,
            projection_bounds_left: 3,
            projection_bounds_right: 4,
        }
    );
    let cbmp = full(b"cbmp", &[0, 0, 0, 0, 0, 0, 0, 2, 7]);
    let decoded = Cbmp::decode(&mut &cbmp[..]).unwrap();
    assert_eq!(
        decoded,
        Cbmp {
            layout: 0,
            padding: 2
        }
    );
    assert_eq!(encode(&decoded), full(b"cbmp", &[0, 0, 0, 0, 0, 0, 0, 2]));
}

#[test]
fn svhd_edge_cases() {
    // Not UTF-8: kept verbatim by its parent.
    let invalid = full(b"svhd", &[0xff, 0xfe, 0]);
    assert!(Svhd::decode(&mut &invalid[..]).is_err());
    round_trip(
        &atom(b"sv3d", &invalid),
        Sv3d {
            unknown: vec![kept(&invalid)],
            ..Default::default()
        },
    );

    // No NUL, or no string at all: read as is, written back with the NUL.
    for (body, source) in [(&b"Tool"[..], "Tool"), (&b""[..], "")] {
        let decoded = Svhd::decode(&mut &full(b"svhd", body)[..]).unwrap();
        assert_eq!(decoded.metadata_source, source);
        assert_eq!(encode(&decoded), full(b"svhd", &[body, b"\0"].concat()));
    }
}

#[test]
fn full_boxes_reject_other_versions_and_flags() {
    fn rejects<T: Atom + std::fmt::Debug>(kind: &[u8; 4], body: &[u8]) {
        for (version, flags) in [(1, 0), (0, 1)] {
            let encoded = full_vf(kind, version, flags, body);
            assert!(
                T::decode(&mut &encoded[..]).is_err(),
                "{kind:?} v{version} f{flags}"
            );
        }
    }
    rejects::<St3d>(b"st3d", &[2]);
    rejects::<Svhd>(b"svhd", b"\0");
    rejects::<Prhd>(b"prhd", &[0; 12]);
    rejects::<Equi>(b"equi", &[0; 16]);
    rejects::<Cbmp>(b"cbmp", &[0; 8]);
    rejects::<Mshp>(b"mshp", &[0; 8]);
    rejects::<Must>(b"must", b"eyes");
    rejects::<Stri>(b"stri", &[3]);
    rejects::<Hero>(b"hero", &[1]);
    rejects::<Blin>(b"blin", &[0; 4]);
    rejects::<Dadj>(b"dadj", &[0; 4]);
    rejects::<Prji>(b"prji", b"hequ");
    rejects::<Pkin>(b"pkin", b"side");
}

#[test]
fn vexu() {
    let encoded = atom(
        b"vexu",
        &[
            full(b"must", b"eyes"),
            atom(
                b"eyes",
                &[
                    full(b"must", b"stri"),
                    full(b"stri", &[0b0011]),
                    full(b"hero", &[1]),
                    atom(b"cams", &full(b"blin", &63_000u32.to_be_bytes())),
                    atom(b"cmfy", &full(b"dadj", &(-200i32).to_be_bytes())),
                ]
                .concat(),
            ),
            atom(b"pack", &full(b"pkin", b"side")),
            atom(
                b"proj",
                &[full(b"prji", b"hequ"), full(b"hequ", &[])].concat(),
            ),
        ]
        .concat(),
    );

    round_trip(
        &encoded,
        Vexu {
            must: Some(Must {
                required_box_types: vec![Eyes::KIND],
            }),
            proj: Some(VexuProj {
                prji: Some(Prji {
                    projection_kind: Prji::HALF_EQUIRECTANGULAR,
                }),
                kind: Some(VexuProjKind::HalfEquirectangular),
                ..Default::default()
            }),
            eyes: Some(Eyes {
                must: Some(Must {
                    required_box_types: vec![Stri::KIND],
                }),
                stri: Some(Stri {
                    has_left_eye_view: true,
                    has_right_eye_view: true,
                    ..Default::default()
                }),
                hero: Some(Hero {
                    hero_eye_indicator: Hero::LEFT,
                }),
                cams: Some(Cams {
                    blin: Some(Blin {
                        baseline_value: 63_000,
                    }),
                    ..Default::default()
                }),
                cmfy: Some(Cmfy {
                    dadj: Some(Dadj {
                        disparity_adjustment: -200,
                    }),
                    ..Default::default()
                }),
                unknown: vec![],
            }),
            pack: Some(Pack {
                pkin: Some(Pkin {
                    view_packing_kind: Pkin::SIDE_BY_SIDE,
                }),
                ..Default::default()
            }),
            unknown: vec![],
        },
    );
}

#[test]
fn vexu_writes_proj_last_and_prji_first() {
    // Apple does not prescribe an order, but a reader may read only the
    // `prji` at the start of `proj`, and take the rest of `proj` for the next
    // sibling.
    let lnsc = atom(b"lnsc", &atom(b"lens", &[]));
    let free = atom(b"free", &[0; 4]);
    let input = atom(
        b"vexu",
        &[
            atom(
                b"proj",
                &[
                    full(b"must", b"prji"),
                    full(b"hequ", &[]),
                    full(b"prji", b"hequ"),
                ]
                .concat(),
            ),
            lnsc.clone(),
            atom(b"pack", &full(b"pkin", b"side")),
            free.clone(),
            atom(b"eyes", &full(b"stri", &[3])),
            full(b"must", b"eyes"),
        ]
        .concat(),
    );
    let vexu = Vexu::decode(&mut input.as_slice()).unwrap();
    assert_eq!(vexu.unknown, vec![kept(&lnsc), kept(&free)]);

    let encoded = encode(&vexu);
    assert_eq!(
        child_kinds(&encoded, 8),
        ["must", "eyes", "pack", "lnsc", "free", "proj"]
    );
    let (_, proj) = children(&encoded, 8).pop().unwrap();
    assert_eq!(child_kinds(proj, 8), ["prji", "hequ", "must"]);
    assert_eq!(Vexu::decode(&mut encoded.as_slice()).unwrap(), vexu);
}

#[test]
fn vexu_optional_children() {
    // `prji`, `blin`, `dadj` and `pkin` are all "Mandatory: No".
    round_trip(
        &atom(
            b"vexu",
            &[
                atom(
                    b"eyes",
                    &[
                        full(b"stri", &[0b0011]),
                        atom(b"cams", &[]),
                        atom(b"cmfy", &[]),
                    ]
                    .concat(),
                ),
                atom(b"pack", &[]),
                atom(b"proj", &[]),
            ]
            .concat(),
        ),
        Vexu {
            proj: Some(VexuProj::default()),
            eyes: Some(Eyes {
                stri: Some(Stri {
                    has_left_eye_view: true,
                    has_right_eye_view: true,
                    ..Default::default()
                }),
                cams: Some(Cams::default()),
                cmfy: Some(Cmfy::default()),
                ..Default::default()
            }),
            pack: Some(Pack::default()),
            ..Default::default()
        },
    );
}

#[test]
fn vexu_projection_kinds() {
    for kind in [
        VexuProjKind::Rectilinear,
        VexuProjKind::Equirectangular,
        VexuProjKind::HalfEquirectangular,
        VexuProjKind::Fisheye,
        VexuProjKind::ParametricImmersive,
    ] {
        let four_cc = *kind.kind().as_ref();
        round_trip(
            &atom(
                b"proj",
                &[full(b"prji", &four_cc), full(&four_cc, &[])].concat(),
            ),
            VexuProj {
                prji: Some(Prji {
                    projection_kind: kind.kind(),
                }),
                kind: Some(kind),
                ..Default::default()
            },
        );
    }
}

#[test]
fn vexu_keeps_unknown_children() {
    let abcd = full(b"abcd", &[1]);
    let lnsc = atom(b"lnsc", &atom(b"lens", &full(b"lnhd", &[0; 4])));
    let free = atom(b"free", &[0; 4]);
    round_trip(
        &atom(
            b"vexu",
            &[
                atom(b"eyes", &[full(b"stri", &[0b0001]), abcd.clone()].concat()),
                lnsc.clone(),
                free.clone(),
            ]
            .concat(),
        ),
        Vexu {
            eyes: Some(Eyes {
                stri: Some(Stri {
                    has_left_eye_view: true,
                    ..Default::default()
                }),
                unknown: vec![kept(&abcd)],
                ..Default::default()
            }),
            unknown: vec![kept(&lnsc), kept(&free)],
            ..Default::default()
        },
    );
}

#[test]
fn vexu_keeps_invalid_children() {
    // Each invalid child arrives with its slot still empty, so it is the check
    // under test that keeps it, not the repeat check.
    for invalid in [
        // "readers should treat any non-zero values as if this box is invalid"
        full(b"stri", &[0xf3]),
        full_vf(b"stri", 1, 0, &[3]),
        full_vf(b"hero", 0, 1, &[1]),
        full(b"hero", &[1, 0]),
    ] {
        round_trip(
            &atom(b"eyes", &invalid),
            Eyes {
                unknown: vec![kept(&invalid)],
                ..Default::default()
            },
        );
    }

    // A repeat is kept too.
    let second_hero = full(b"hero", &[2]);
    round_trip(
        &atom(
            b"eyes",
            &[full(b"hero", &[1]), second_hero.clone()].concat(),
        ),
        Eyes {
            hero: Some(Hero {
                hero_eye_indicator: Hero::LEFT,
            }),
            unknown: vec![kept(&second_hero)],
            ..Default::default()
        },
    );

    // A projection-kind box with fields this crate does not know, then a second
    // projection-kind box after a valid one.
    let hequ_with_fields = full(b"hequ", &[1, 2, 3, 4]);
    round_trip(
        &atom(
            b"proj",
            &[full(b"prji", b"hequ"), hequ_with_fields.clone()].concat(),
        ),
        VexuProj {
            prji: Some(Prji {
                projection_kind: Prji::HALF_EQUIRECTANGULAR,
            }),
            unknown: vec![kept(&hequ_with_fields)],
            ..Default::default()
        },
    );
    let second_kind = full(b"equi", &[]);
    round_trip(
        &atom(
            b"proj",
            &[
                full(b"prji", b"rect"),
                full(b"rect", &[]),
                second_kind.clone(),
            ]
            .concat(),
        ),
        VexuProj {
            prji: Some(Prji {
                projection_kind: Prji::RECTILINEAR,
            }),
            kind: Some(VexuProjKind::Rectilinear),
            unknown: vec![kept(&second_kind)],
            ..Default::default()
        },
    );

    // A `must` whose length is not a multiple of four.
    let short_must = full(b"must", b"eyes!");
    round_trip(
        &atom(b"vexu", &short_must),
        Vexu {
            unknown: vec![kept(&short_must)],
            ..Default::default()
        },
    );
}

#[test]
fn kept_boxes_stay_in_their_slot() {
    // A reader may expect `svhd` first in `sv3d`, `prhd` first in `proj`,
    // `prji` first in Apple's `proj`, and `st3d` before `vexu`. A kept box goes
    // back where its kind goes, even when its slot is empty.
    let bad_svhd = full(b"svhd", &[0xe9, 0]);
    let proj = atom(
        b"proj",
        &[full(b"prhd", &[0; 12]), full(b"equi", &[0; 16])].concat(),
    );
    let decoded =
        Sv3d::decode(&mut atom(b"sv3d", &[bad_svhd.clone(), proj.clone()].concat()).as_slice())
            .unwrap();
    assert_eq!(decoded.unknown, vec![kept(&bad_svhd)]);
    assert_eq!(encode(&decoded), atom(b"sv3d", &[bad_svhd, proj].concat()));

    let bad_prhd = full_vf(b"prhd", 0, 1, &[0; 12]);
    let input = atom(
        b"proj",
        &[bad_prhd.clone(), full(b"equi", &[0; 16])].concat(),
    );
    let decoded = Proj::decode(&mut input.as_slice()).unwrap();
    assert_eq!(decoded.unknown, vec![kept(&bad_prhd)]);
    assert_eq!(encode(&decoded), input);

    let bad_prji = full_vf(b"prji", 0, 1, b"hequ");
    let input = atom(b"proj", &[bad_prji.clone(), full(b"hequ", &[])].concat());
    let decoded = VexuProj::decode(&mut input.as_slice()).unwrap();
    assert_eq!(decoded.unknown, vec![kept(&bad_prji)]);
    assert_eq!(encode(&decoded), input);

    let bad_st3d = full_vf(b"st3d", 0, 1, &[2]);
    let vexu = atom(b"vexu", &atom(b"eyes", &full(b"stri", &[3])));
    let entry = hvc1_with(&[bad_st3d.clone(), vexu].concat());
    let hvc1 = Hvc1::decode(&mut entry.as_slice()).unwrap();
    let spatial = hvc1.spatial.as_deref().unwrap();
    assert_eq!(spatial.st3d, None);
    assert_eq!(spatial.unknown, vec![kept(&bad_st3d)]);
    assert_eq!(encode(&hvc1), entry);
}

#[test]
fn child_header_edge_cases() {
    let svhd = full(b"svhd", b"\0");
    let proj = atom(b"proj", &full(b"prhd", &[0; 12]));
    let expected = Sv3d {
        svhd: Some(Svhd::default()),
        proj: Some(Proj {
            prhd: Some(Prhd::default()),
            ..Default::default()
        }),
        unknown: vec![],
    };
    let explicit = atom(b"sv3d", &[svhd.clone(), proj.clone()].concat());

    // A size-0 child runs to the end of its parent; a 64-bit size is read too.
    // Both are written back with an explicit 32-bit size.
    let size0 = [&[0, 0, 0, 0][..], b"proj", &proj[8..]].concat();
    let large = [
        &[0, 0, 0, 1][..],
        b"proj",
        &((proj.len() + 8) as u64).to_be_bytes(),
        &proj[8..],
    ]
    .concat();
    for child in [size0, large] {
        let decoded =
            Sv3d::decode(&mut atom(b"sv3d", &[svhd.clone(), child].concat()).as_slice()).unwrap();
        assert_eq!(decoded, expected);
        assert_eq!(encode(&decoded), explicit);
    }

    // Fewer than 8 bytes left is padding, and is dropped.
    let padded = atom(b"sv3d", &[svhd.clone(), proj.clone(), vec![0; 4]].concat());
    assert_eq!(
        encode(&Sv3d::decode(&mut padded.as_slice()).unwrap()),
        explicit
    );

    // Eight zero bytes read as a size-0 box of type 0, kept verbatim.
    let zeros = atom(b"sv3d", &[svhd, proj, vec![0; 8]].concat());
    let decoded = Sv3d::decode(&mut zeros.as_slice()).unwrap();
    assert_eq!(decoded.unknown, vec![Any::Unknown(FourCC::from(0), vec![])]);

    // A child header with a size below 8 fails its container, which its
    // parent keeps verbatim.
    let eyes = atom(b"eyes", &[0, 0, 0, 4, b'a', b'b', b'c', b'd']);
    round_trip(
        &atom(b"vexu", &eyes),
        Vexu {
            unknown: vec![kept(&eyes)],
            ..Default::default()
        },
    );
}

#[test]
fn hfov() {
    round_trip(
        &atom(b"hfov", &90_000u32.to_be_bytes()),
        Hfov {
            field_of_view: 90_000,
        },
    );
}

#[test]
fn sample_entry() {
    let hvc1 = Hvc1 {
        btrt: Some(Btrt {
            buffer_size_db: 0,
            max_bitrate: 8_000_000,
            avg_bitrate: 6_000_000,
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
        spatial: Some(Box::new(Spatial {
            st3d: Some(St3d {
                stereo_mode: St3d::TOP_BOTTOM,
            }),
            sv3d: Some(Sv3d {
                proj: Some(Proj {
                    prhd: Some(Prhd::default()),
                    kind: Some(ProjKind::Equi(Equi::default())),
                    unknown: vec![],
                }),
                ..Default::default()
            }),
            vexu: Some(Vexu {
                proj: Some(VexuProj {
                    prji: Some(Prji {
                        projection_kind: Prji::RECTILINEAR,
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            hfov: Some(Hfov {
                field_of_view: 63_400,
            }),
            unknown: vec![],
        })),
        ..Default::default()
    };

    let buf = encode(&hvc1);
    assert_eq!(Hvc1::decode(&mut buf.as_slice()).unwrap(), hvc1);

    // Spherical Video V2: `st3d` then `sv3d`, before `pasp`; `st3d` also goes
    // before `vexu` and `hfov`, as a reader may refuse an `st3d` after them.
    assert_eq!(
        child_kinds(&buf, ENTRY_HEADER),
        ["hvcC", "btrt", "colr", "st3d", "sv3d", "vexu", "hfov", "pasp"]
    );
}

#[test]
fn sample_entry_without_spatial_boxes() {
    let entry = encode(&Hvc1::default());
    assert_eq!(Hvc1::decode(&mut entry.as_slice()).unwrap().spatial, None);
}

#[test]
fn sample_entry_keeps_invalid_spatial_boxes() {
    // None of these may fail the sample entry: each is kept verbatim, at the
    // position of its kind.
    let second_st3d = full(b"st3d", &[2]);
    // A child whose size runs past the end of `vexu`.
    let broken_vexu = atom(b"vexu", &[0, 0, 0xff, 0xff, b'e', b'y', b'e', b's']);
    let short_hfov = atom(b"hfov", &[0, 1]);
    let short_prhd = full(b"prhd", &[0; 4]);

    let entry = hvc1_with(
        &[
            full(b"st3d", &[0]),
            second_st3d.clone(),
            atom(b"sv3d", &atom(b"proj", &short_prhd)),
            broken_vexu.clone(),
            short_hfov.clone(),
        ]
        .concat(),
    );

    let hvc1 = Hvc1::decode(&mut entry.as_slice()).unwrap();
    assert_eq!(
        hvc1.spatial.as_deref(),
        Some(&Spatial {
            st3d: Some(St3d { stereo_mode: 0 }),
            sv3d: Some(Sv3d {
                proj: Some(Proj {
                    unknown: vec![kept(&short_prhd)],
                    ..Default::default()
                }),
                ..Default::default()
            }),
            vexu: None,
            hfov: None,
            unknown: vec![kept(&second_st3d), kept(&broken_vexu), kept(&short_hfov)],
        })
    );
    assert_eq!(encode(&hvc1), entry);
}

#[test]
fn writer_vr180_side_by_side() {
    // What vr180stab writes for a 180° left-right half-equirect export: each
    // eye covers the middle half of the 360° projection.
    let st3d = St3d {
        stereo_mode: St3d::LEFT_RIGHT,
    };
    let sv3d = Sv3d {
        svhd: Some(Svhd {
            metadata_source: "vr180stab".into(),
        }),
        proj: Some(Proj {
            prhd: Some(Prhd::default()),
            kind: Some(ProjKind::Equi(Equi {
                projection_bounds_left: 0x4000_0000,
                projection_bounds_right: 0x4000_0000,
                ..Default::default()
            })),
            ..Default::default()
        }),
        ..Default::default()
    };
    // The bytes in a real vr180stab export.
    let hex = |s: &str| {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>()
    };
    assert_eq!(encode(&st3d), hex("0000000d737433640000000002"));
    assert_eq!(
        encode(&sv3d),
        hex(concat!(
            "0000005a73763364000000167376686400000000767231383073746162000000",
            "003c70726f6a0000001870726864000000000000000000000000000000000000",
            "001c657175690000000000000000000000004000000040000000"
        ))
    );

    // The Apple equivalent, for visionOS (APMP for production).
    let vexu = Vexu {
        eyes: Some(Eyes {
            stri: Some(Stri {
                has_left_eye_view: true,
                has_right_eye_view: true,
                ..Default::default()
            }),
            ..Default::default()
        }),
        pack: Some(Pack {
            pkin: Some(Pkin {
                view_packing_kind: Pkin::SIDE_BY_SIDE,
            }),
            ..Default::default()
        }),
        proj: Some(VexuProj {
            prji: Some(Prji {
                projection_kind: Prji::HALF_EQUIRECTANGULAR,
            }),
            kind: Some(VexuProjKind::HalfEquirectangular),
            ..Default::default()
        }),
        ..Default::default()
    };
    let spatial = Spatial {
        st3d: Some(st3d),
        sv3d: Some(sv3d),
        vexu: Some(vexu),
        hfov: Some(Hfov {
            field_of_view: 180 * Hfov::DENOMINATOR,
        }),
        unknown: vec![],
    };
    let encoded = encode(&spatial);
    assert_eq!(child_kinds(&encoded, 0), ["st3d", "sv3d", "vexu", "hfov"]);
    let vexu = &encoded[13 + 90..encoded.len() - 12];
    assert_eq!(child_kinds(vexu, 8), ["eyes", "pack", "proj"]);
}

#[cfg(feature = "serde")]
#[test]
fn serde_fields_default() {
    use serde_test::{assert_de_tokens, Token};

    // Fields left out of the spatial containers take their defaults.
    assert_de_tokens(
        &Spatial {
            st3d: Some(St3d { stereo_mode: 2 }),
            ..Default::default()
        },
        &[
            Token::Struct {
                name: "Spatial",
                len: 1,
            },
            Token::Str("st3d"),
            Token::Some,
            Token::Struct {
                name: "St3d",
                len: 1,
            },
            Token::Str("stereo_mode"),
            Token::U8(2),
            Token::StructEnd,
            Token::StructEnd,
        ],
    );
    assert_de_tokens(
        &Vexu {
            eyes: Some(Eyes {
                stri: Some(Stri {
                    has_left_eye_view: true,
                    has_right_eye_view: true,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        },
        &[
            Token::Struct {
                name: "Vexu",
                len: 1,
            },
            Token::Str("eyes"),
            Token::Some,
            Token::Struct {
                name: "Eyes",
                len: 1,
            },
            Token::Str("stri"),
            Token::Some,
            Token::Struct {
                name: "Stri",
                len: 4,
            },
            Token::Str("eye_views_reversed"),
            Token::Bool(false),
            Token::Str("has_additional_views"),
            Token::Bool(false),
            Token::Str("has_right_eye_view"),
            Token::Bool(true),
            Token::Str("has_left_eye_view"),
            Token::Bool(true),
            Token::StructEnd,
            Token::StructEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn spatial_boxes_are_not_in_any() {
    // These only mean something inside a sample entry, so elsewhere they stay
    // unknown, exactly as before they were modelled.
    for kind in [
        b"st3d", b"sv3d", b"svhd", b"proj", b"prhd", b"equi", b"cbmp", b"mshp", b"vexu", b"must",
        b"prji", b"hequ", b"eyes", b"stri", b"hero", b"cams", b"blin", b"cmfy", b"dadj", b"pack",
        b"pkin", b"hfov",
    ] {
        let decoded = Any::decode(&mut atom(kind, &[1, 2]).as_slice()).unwrap();
        assert_eq!(decoded, Any::Unknown(FourCC::new(kind), vec![1, 2]));
    }
}
