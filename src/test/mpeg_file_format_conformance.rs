use std::path::PathBuf;

use crate::{Any, ReadFrom};

#[test]
fn test_published() {
    let expected_fails: Vec<String> = vec![
        "FileFormatConformance/data/file_features/published/isobmff/02_dref_edts_img.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/timed-metadata.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/22_tx3g.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/a7-tone-oddities.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/04_bifs_video.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/09_text.mp4".into(),
    ];

    let paths =
        std::fs::read_dir("FileFormatConformance/data/file_features/published/isobmff/").unwrap();
    for path in paths {
        let direntry = path.unwrap();
        let path = direntry.path().into_os_string().into_string().unwrap();
        if path.ends_with(".mp4") {
            println!("checking {:?}", direntry);
            match check_one_file(&direntry.path()) {
                true => assert!(
                    !expected_fails.contains(&path),
                    "expected {path} to fail, but it unexpectedly passed"
                ),
                false => assert!(
                    expected_fails.contains(&path),
                    "expected {path} to pass, but it unexpectedly failed"
                ),
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
