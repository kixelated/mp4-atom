use crate::*;

use std::io::Cursor;

#[test]
fn read_c2pa_from_c2pa_mp4() -> Result<()> {
    // ./c2pa.mp4 is ./av1.mp4 signed with c2patool.
    // It has been independently verified with bento4 - mp4dump for its structure
    const ENCODED: &[u8] = include_bytes!("c2pa.mp4");

    let mut cursor = Cursor::new(ENCODED);

    let received_ftyp = Ftyp::decode(&mut cursor)?;
    let expected_ftyp = Ftyp {
        major_brand: b"iso6".into(),
        minor_version: 0x200, // 512 in decimal
        compatible_brands: vec![
            b"iso6".into(),
            b"cmfc".into(),
            b"av01".into(),
            b"mp41".into(),
        ],
    };

    assert_eq!(received_ftyp, expected_ftyp);

    let uuid = Uuid::decode(&mut cursor)?;

    let c2pa_data = match uuid {
        Uuid::C2pa(data) => data,
        other => panic!("Expected Uuid::C2pa, got {:?}", other),
    };

    assert_eq!(c2pa_data.box_purpose, "manifest");

    Ok(())
}
