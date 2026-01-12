use std::path::{Path, PathBuf};

use crate::{Any, ReadFrom};

#[test]
fn test_published() {
    let expected_fails: Vec<String> = vec![
        "FileFormatConformance/data/file_features/published/isobmff/02_dref_edts_img.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/timed-metadata.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/a7-tone-oddities.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/04_bifs_video.mp4".into(),
        "FileFormatConformance/data/file_features/published/green/video_2500000bps_0.mp4".into(),
        "FileFormatConformance/data/file_features/published/heif/C027.heic".into(),
        "FileFormatConformance/data/file_features/published/heif/C028.heic".into(),
        "FileFormatConformance/data/file_features/published/heif/C041.heic".into(),
        "FileFormatConformance/data/file_features/published/isobmff/compact-no-code-fec-1.iso3"
            .into(),
        "FileFormatConformance/data/file_features/published/isobmff/compact-no-code-fec-2.iso3"
            .into(),
        "FileFormatConformance/data/file_features/published/isobmff/mbms-fec.iso3".into(),
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
