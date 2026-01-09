use std::path::PathBuf;

use crate::{Any, ReadFrom};

#[test]
fn test_published() {
    let suppressed: Vec<String> = vec![
        "FileFormatConformance/data/file_features/published/isobmff/fragment_random_access-2.mp4"
            .into(),
        "FileFormatConformance/data/file_features/published/isobmff/02_dref_edts_img.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/FX-VY-9436R.3_qhd-variant.mp4"
            .into(),
        "FileFormatConformance/data/file_features/published/isobmff/FX-VY-9436R.3_qhd.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/timed-metadata.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/22_tx3g.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/a7-tone-oddities.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/04_bifs_video.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/fragment-random-access-1+AF8-rev1.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/sg-tl-st.mp4".into(),
        "FileFormatConformance/data/file_features/published/isobmff/09_text.mp4".into(),
    ];

    let paths =
        std::fs::read_dir("FileFormatConformance/data/file_features/published/isobmff/").unwrap();
    for path in paths {
        let direntry = path.unwrap();
        let path = direntry.path().into_os_string().into_string().unwrap();
        if path.ends_with(".mp4") && !suppressed.contains(&path) {
            println!("checking {:?}", direntry);
            check_one_file(&direntry.path());
        }
    }
}

fn check_one_file(path: &PathBuf) {
    let mut input = std::fs::File::open(path).unwrap();
    while let Some(atom) = Option::<Any>::read_from(&mut input).unwrap() {
        if let Any::Unknown(kind, data) = atom {
            panic!("Unknown {{ kind: {:?}, size: {:?} }}", kind, data.len());
        }
    }
}
