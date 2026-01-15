use std::path::{Path, PathBuf};

use crate::{Any, ReadFrom};

#[test]
fn test_published() {
    let expected_fails: Vec<String> = vec![

        "FileFormatConformance/data/file_features/published/3gp/pdin_example.3gp".into(),
        "FileFormatConformance/data/file_features/published/3gp/female_amr67DTX_hinted.3gp".into(),
        "FileFormatConformance/data/file_features/published/3gp/female_amr67_hinted.3gp".into(),
        "FileFormatConformance/data/file_features/published/3gp/male_amr122.3gp".into(),
        "FileFormatConformance/data/file_features/published/3gp/male_amr122DTX.3gp".into(),
        "FileFormatConformance/data/file_features/published/3gp/rs_example_r1.3gp".into(),
        "FileFormatConformance/data/file_features/published/mpeg-audio-conformance/ac01.mp4".into(),
        "FileFormatConformance/data/file_features/published/mpeg-audio-conformance/sls2100_aot02_048_16.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/01_simple.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/02_dref_edts_img.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/03_hinted.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/06_bifs.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/07_bifs_sprite.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/timed-metadata.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/a5-foreman-AVC.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/a6_tone_multifile.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/a7-tone-oddities.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/04_bifs_video.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/10_fragments.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/12_metas_v2.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/13_long.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/FX-VY-9436R.3_qhd-variant.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/FX-VY-9436R.3_qhd.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/05_bifs_video_protected_v2.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/20_stxt.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/a4-tone-fragmented.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/21_segment.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/sg-tl-st.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/restricted.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/17_negative_ctso.mp4".into(),
        "FileFormatConformance/data/file_features/published/green/meta_2500000bps_0.mp4m".into(),
        "FileFormatConformance/data/file_features/published/green/video_2500000bps_0.mp4".into(),
        "FileFormatConformance/data/file_features/published/heif/C027.heic".into(),
        "FileFormatConformance/data/file_features/published/heif/C028.heic".into(),
        "FileFormatConformance/data/file_features/published/heif/C032.heic".into(),
        "FileFormatConformance/data/file_features/published/heif/C041.heic".into(),
        "FileFormatConformance/data/file_features/published/isobmff/compact-no-code-fec-1.iso3"
            .into(),
        "FileFormatConformance/data/file_features/published/isobmff/compact-no-code-fec-2.iso3"
            .into(),
        "FileFormatConformance/data/file_features/published/isobmff/mbms-fec.iso3".into(),
        "FileFormatConformance/data/file_features/published/isobmff/fragment_random_access-2.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/fragment-random-access-1+AF8-rev1.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/a9-aac-samplegroups-edit.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/a1-foreman-QCIF.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/a2-foreman-QCIF-hinted.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/a10-foreman_QCIF-raw.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/a3-tone-protected.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/a3b-tone-deprot.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/f1.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/f2.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/08_bifs_carousel_v2.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/16_vtt.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/19_ttml.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/18_pssh_v2.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/a8-foreman_QCIF_edit.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/rtp_rtcp_reception_hint_tracks_v2.mp4".into(),
        "FileFormatConformance/data/file_features/published/maf/vsaf/1.mp4".into(),
        "FileFormatConformance/data/file_features/published/uvvu/Solekai002_1280_23_1x1_v7clear.uvvu".into(),
        "FileFormatConformance/data/file_features/published/uvvu/Solekai007_1920_29_1x1_v7clear.uvu".into(),
        "FileFormatConformance/data/file_features/published/nalu/hevc/hevc_tiles_single_track_trif_full_picture.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/hevc/alst_hvc1.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/hevc/hevc_tiles_multiple_tracks.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/hevc/hevc_tiles_single_track_nalm_rle.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/hevc/trgr_hvc1.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/hevc/hvc2_extractors.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/hevc/aggr_hvc1.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/hevc/hevc_hvc1_hvc2_extractors.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/hevc/hevc_hvc1_hvc2_implicit.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/hevc/hevc_hev1_hev2_extractors.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/hevc/hevc_hev1_hev2_implicit.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/hevc/hevc_tiles_single_track_nalm.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/hevc/subs_tile_hvc1.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/hevc/subs_slice_hvc1.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/hevc/hevc_tiles_single_track_nalm_all_intra.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/hevc/hevc_tiles_multiple_tracks_empty_base.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/svc/mp4-live-LastTime-depRep.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/mvc/DDF_10s_25fps.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/mvc/DDF_10s_25fps-dynamic.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/l-hevc/shvc_hvc2_single_track.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/l-hevc/mhvc_hvc1_hvc2_multiple_tracks_extractors.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/l-hevc/shvc_hev2_single_track.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/l-hevc/lhevc_avc3_lhe1.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/l-hevc/shvc_hvc1_lhv1_multiple_tracks_implicit.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/l-hevc/lhevc_avc3_lhv1.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/l-hevc/shvc_hev1_single_track.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/l-hevc/mhvc_hvc1_single_track.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/l-hevc/lhevc_avc1_lhe1.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/l-hevc/mhvc_hev1_lhe1_multiple_tracks_implicit.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/l-hevc/mhvc_hvc1_lhv1_multiple_tracks_implicit.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/l-hevc/mhvc_hev1_single_track.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/l-hevc/shvc_hev1_hev2_multiple_tracks_extractors.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/l-hevc/mhvc_hev1_hev2_multiple_tracks_extractors.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/l-hevc/lhevc_avc1_lhv1.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/l-hevc/shvc_hvc1_single_track.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/l-hevc/mhvc_hev2_single_track.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/l-hevc/shvc_hev1_lhe1_multiple_tracks_implicit.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/l-hevc/shvc_hvc1_hvc2_multiple_tracks_extractors.mp4".into(),
        "FileFormatConformance/data/file_features/published/nalu/l-hevc/mhvc_hvc2_single_track.mp4".into(),

    ];

    for entry in std::fs::read_dir("FileFormatConformance/data/file_features/published").unwrap() {
        let direntry = entry.unwrap();
        let path = direntry.path();
        if path.is_dir() {
            check_directory(&expected_fails, &path);
        }
    }
}

fn check_directory(expected_fails: &Vec<String>, directory: &Path) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let direntry = entry.unwrap();
        let path = direntry.path();
        if path.is_dir() {
            check_directory(expected_fails, &path);
        } else {
            let filepath = direntry.path().into_os_string().into_string().unwrap();
            if !filepath.ends_with(".json")
                && !filepath.ends_with(".dat")
                && !filepath.ends_with(".zip")
                && !filepath.ends_with(".txt")
                && !filepath.ends_with(".xml")
            {
                println!("checking {:?}", direntry);
                match check_one_file(&direntry.path()) {
                    true => assert!(
                        !expected_fails.contains(&filepath),
                        "expected {filepath} to fail, but it unexpectedly passed"
                    ),
                    false => assert!(
                        expected_fails.contains(&filepath),
                        "expected {filepath} to pass, but it unexpectedly failed"
                    ),
                }
            }
        }
    }
}

fn check_one_file(path: &PathBuf) -> bool {
    let mut input = std::fs::File::open(path).unwrap();
    let mut full_parse = true;
    loop {
        let parse_result = Option::<Any>::read_from(&mut input);
        match parse_result {
            Ok(maybe_atom) => match maybe_atom {
                Some(_) => {}
                None => {
                    break;
                }
            },
            Err(err) => {
                println!("{err:#?}");
                full_parse = false;
                break;
            }
        }
    }
    full_parse
}
