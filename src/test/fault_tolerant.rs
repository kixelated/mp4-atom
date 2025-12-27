use crate::*;

/// Test that fault-tolerant feature works as expected
/// When enabled: unexpected boxes are collected
/// When disabled: unexpected boxes cause an error
#[test]
fn test_fault_tolerant_behavior() {
    // Create a buffer with a Moov box containing an unexpected child box
    // Moov structure: mvhd (required) + an unknown/unexpected box
    let mut buf = Vec::new();

    // First encode a valid Moov with mvhd
    let moov = Moov {
        mvhd: Mvhd {
            creation_time: 0,
            modification_time: 0,
            timescale: 1000,
            duration: 0,
            rate: 1.into(),
            volume: 1.into(),
            next_track_id: 1,
            ..Default::default()
        },
        meta: None,
        mvex: None,
        trak: vec![],
        udta: None,
        #[cfg(feature = "fault-tolerant")]
        unexpected: vec![],
    };

    // Encode the moov
    moov.encode(&mut buf).unwrap();

    // Now manually insert an unexpected box into the moov
    // We'll use 'mdat' which is a known box type but NOT expected inside moov
    // This should trigger the unexpected box behavior
    let mut moov_buf = Vec::new();

    // Encode mvhd
    moov.mvhd.encode(&mut moov_buf).unwrap();

    // Add an mdat box (which is NOT a child of moov)
    // This should be treated as unexpected
    let mdat_data = vec![
        0x00, 0x00, 0x00, 0x10, // size: 16 bytes
        b'm', b'd', b'a', b't', // type: 'mdat'
        0x01, 0x02, 0x03, 0x04, // some data
        0x05, 0x06, 0x07, 0x08,
    ];
    moov_buf.extend_from_slice(&mdat_data);

    // Now wrap it in a moov box
    let mut final_buf = Vec::new();
    let moov_size = (moov_buf.len() + 8) as u32; // +8 for size and type
    final_buf.extend_from_slice(&moov_size.to_be_bytes());
    final_buf.extend_from_slice(b"moov");
    final_buf.extend_from_slice(&moov_buf);

    // Try to decode the moov
    let result = Moov::decode(&mut final_buf.as_slice());

    #[cfg(feature = "fault-tolerant")]
    {
        // With fault-tolerant feature enabled, it should succeed
        let decoded = result.expect("should decode successfully with fault-tolerant feature");

        // Check that the unexpected box was collected
        assert_eq!(decoded.unexpected.len(), 1, "should have 1 unexpected box");

        // Verify the unexpected box
        match &decoded.unexpected[0] {
            Any::Mdat(mdat) => {
                // The mdat box should be collected as unexpected
                assert_eq!(mdat.data.len(), 8, "mdat data should be 8 bytes");
            }
            _ => panic!(
                "unexpected box should be Any::Mdat variant, got: {:?}",
                decoded.unexpected[0]
            ),
        }

        // Other fields should be decoded correctly
        assert_eq!(decoded.mvhd.timescale, 1000);
        assert_eq!(decoded.trak.len(), 0);
    }

    #[cfg(not(feature = "fault-tolerant"))]
    {
        // Without fault-tolerant feature, it should return an error
        assert!(
            result.is_err(),
            "should fail to decode without fault-tolerant feature"
        );

        // Check that it's specifically an UnexpectedBox error
        match result.unwrap_err() {
            Error::UnexpectedBox(kind) => {
                assert_eq!(
                    kind,
                    FourCC::new(b"mdat"),
                    "error should report 'mdat' as unexpected"
                );
            }
            other => panic!("expected UnexpectedBox error, got: {:?}", other),
        }
    }
}

/// Test that multiple unexpected boxes are collected correctly
#[test]
#[cfg(feature = "fault-tolerant")]
fn test_multiple_unexpected_boxes() {
    // Create a moov with multiple unexpected boxes
    let mut moov_buf = Vec::new();

    // Encode mvhd
    let mvhd = Mvhd {
        creation_time: 0,
        modification_time: 0,
        timescale: 1000,
        duration: 0,
        rate: 1.into(),
        volume: 1.into(),
        next_track_id: 1,
        ..Default::default()
    };
    mvhd.encode(&mut moov_buf).unwrap();

    // Add first unexpected box (ftyp - not expected in moov)
    let unexpected1 = vec![
        0x00, 0x00, 0x00, 0x14, // size: 20 bytes
        b'f', b't', b'y', b'p', // type: 'ftyp'
        b'i', b's', b'o', b'm', // major brand
        0x00, 0x00, 0x02, 0x00, // minor version
        b'm', b'p', b'4', b'1', // compatible brand
    ];
    moov_buf.extend_from_slice(&unexpected1);

    // Add second unexpected box (mdat - not expected in moov)
    let unexpected2 = vec![
        0x00, 0x00, 0x00, 0x10, // size: 16 bytes
        b'm', b'd', b'a', b't', // type: 'mdat'
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    ];
    moov_buf.extend_from_slice(&unexpected2);

    // Wrap in moov box
    let mut final_buf = Vec::new();
    let moov_size = (moov_buf.len() + 8) as u32;
    final_buf.extend_from_slice(&moov_size.to_be_bytes());
    final_buf.extend_from_slice(b"moov");
    final_buf.extend_from_slice(&moov_buf);

    // Decode
    let decoded = Moov::decode(&mut final_buf.as_slice())
        .expect("should decode successfully with multiple unexpected boxes");

    // Verify both unexpected boxes were collected
    assert_eq!(
        decoded.unexpected.len(),
        2,
        "should have 2 unexpected boxes"
    );

    match &decoded.unexpected[0] {
        Any::Ftyp(ftyp) => {
            assert_eq!(ftyp.major_brand, FourCC::new(b"isom"));
        }
        _ => panic!(
            "first unexpected box should be Any::Ftyp, got: {:?}",
            decoded.unexpected[0]
        ),
    }

    match &decoded.unexpected[1] {
        Any::Mdat(mdat) => {
            assert_eq!(mdat.data.len(), 8);
        }
        _ => panic!(
            "second unexpected box should be Any::Mdat, got: {:?}",
            decoded.unexpected[1]
        ),
    }
}

/// Test that unexpected boxes are not encoded back
#[test]
#[cfg(feature = "fault-tolerant")]
fn test_unexpected_boxes_not_encoded() {
    // Create a moov with an unexpected box
    let moov = Moov {
        mvhd: Mvhd {
            creation_time: 0,
            modification_time: 0,
            timescale: 1000,
            duration: 0,
            rate: 1.into(),
            volume: 1.into(),
            next_track_id: 1,
            ..Default::default()
        },
        meta: None,
        mvex: None,
        trak: vec![],
        udta: None,
        unexpected: vec![Any::Mdat(Mdat {
            data: vec![0x01, 0x02, 0x03, 0x04],
        })],
    };

    // Encode the moov
    let mut buf = Vec::new();
    moov.encode(&mut buf).unwrap();

    // Decode it back
    let decoded = Moov::decode(&mut buf.as_slice()).expect("should decode");

    // The unexpected box should NOT be present in the decoded version
    // because it wasn't encoded
    assert_eq!(
        decoded.unexpected.len(),
        0,
        "unexpected boxes should not be encoded"
    );
}
