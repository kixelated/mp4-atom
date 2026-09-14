use crate::*;

use std::io::Cursor;

#[test]
fn read_c2pa_from_c2pa_mp4() -> Result<()> {
    // Example file from c2pa, and independently verified with bento4 - mp4dump and c2patool
    // https://github.com/c2pa-org/public-testfiles/tree/main/legacy/1.4/video/mp4
    const ENCODED: &[u8] = include_bytes!("c2pa.mp4");

    let mut cursor = Cursor::new(ENCODED);

    let recieved_ftyp = Ftyp::decode(&mut cursor)?;
    let expected_ftyp = Ftyp {
        major_brand: b"mp42".into(),
        minor_version: 0,
        compatible_brands: vec![b"isom".into(), b"mp42".into()],
    };

    assert_eq!(recieved_ftyp, expected_ftyp);

    let uuid = Uuid::decode(&mut cursor)?;

    let c2pa_data = match uuid {
        Uuid::C2pa(data) => data,
        other => panic!("Expected Uuid::C2pa, got {:?}", other),
    };

    assert_eq!(c2pa_data.box_purpose, "manifest");

    Ok(())
}
