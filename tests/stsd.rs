#![cfg(not(feature = "strict"))]

use mp4_atom::{Codec, Decode, Encode, FourCC, Stsd};

#[test]
fn unknown_codec_round_trips() {
    let input = b"\0\0\0\x1cstsd\0\0\0\0\0\0\0\x01\0\0\0\x0cdvh1\x01\x02\x03\x04";
    let mut buf = input.as_slice();

    let stsd = Stsd::decode(&mut buf).unwrap();
    assert_eq!(
        stsd.codecs,
        vec![Codec::Unknown(FourCC::new(b"dvh1"), vec![1, 2, 3, 4])]
    );

    let mut output = Vec::new();
    stsd.encode(&mut output).unwrap();

    assert_eq!(output, input);
}
