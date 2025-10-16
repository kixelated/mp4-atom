[![crates.io](https://img.shields.io/crates/v/mp4-atom)](https://crates.io/crates/mp4-atom)
[![docs.rs](https://img.shields.io/docsrs/mp4-atom)](https://docs.rs/mp4-atom)
[![discord](https://img.shields.io/discord/1124083992740761730)](https://discord.gg/FCYF3p99mr)

# mp4-atom
This library provides encoding for the ISO Base Media File Format (ISO/IEC 14496-12).
It's meant to be low level, performing encoding/decoding of the binary format without
validation or interpretation of the data. You have to know what boxes to expect!

## Atoms
MP4 files are made up of atoms, which are boxes of data.
They have an upfront size and a 4-byte code to identify the type of box.
Examples include `moov`, `mdat`, `trak`, etc.

Unfortunately, the specification is quite complex and often gated behind a paywall.
Using this library does require some additional knowledge of the format otherwise you should use a higher level library.

See the [documentation](https://docs.rs/mp4-atom).

## Fault-Tolerant Parsing
Enable the `fault-tolerant` feature to support parsing files with unexpected boxes.

```toml
[dependencies]
mp4-atom = { version = "0.9", features = ["fault-tolerant"] }
```

When this feature is enabled, if a container box (such as `moov`, `trak`, `mdia`, etc.) encounters an unexpected child box during decoding, instead of returning an error, the unknown box is collected in an `unexpected: Vec<Any>` field.

When the feature is **disabled** (default), encountering an unexpected box will return an `Error::UnexpectedBox` error, ensuring strict compliance with the expected structure.

Note that when encoding, the `unexpected` boxes are **not** written back - only the explicitly defined fields are encoded.

## Examples
### Decoding/encoding a byte buffer
```rust
use bytes::{Bytes, BufMut};
use mp4_atom::{Any, Encode, Decode, Ftyp};

 // A simple ftyp atom
let mut input = Bytes::from_static(b"\0\0\0\x14ftypiso6\0\0\x02\0mp41");
let atom = Any::decode(&mut input.clone())?;

// Make sure we got the right atom
assert_eq!(atom, Ftyp {
   major_brand: b"iso6".into(),
   minor_version: 512,
   compatible_brands: vec![b"mp41".into()],
}.into());

// Encode it back
let mut output = BufMut::new();
atom.encode(&mut output)?;

assert_eq!(input, output.freeze());
```

### Synchronous IO
NOTE: reading a `Mdat` atom will read the entire contents into memory.
See the next example to avoid this.

```rust
use mp4_atom::{Any, ReadFrom, WriteTo, Ftyp};

let mut reader = std::io::stdin();
let atom = Any::read_from(&mut reader)?;

// Make sure we got the right atom
assert_eq!(atom, Ftyp {
   major_brand: b"iso6".into(),
   minor_version: 512,
   compatible_brands: vec![b"mp41".into()],
}.into());

// Encode it back to a Write type
let writer = std::io::stdout();
atom.write_to(&mut writer)?;
```

### Handling large atoms
To avoid reading large files into memory, you can call `Header::read_from` manually:

```rust
use mp4_atom::{Atom, Any, Header, ReadFrom, ReadAtom, WriteTo, Ftyp, Moov};

let mut reader = std::io::stdin();

let header = Header::read_from(&mut reader)?;
match header.kind {
  Ftyp::KIND => {
    let ftyp = Ftyp::read_atom(&header, &mut reader)?;

     // Make sure we got the right atom
     assert_eq!(ftyp, Ftyp {
       major_brand: b"iso6".into(),
       minor_version: 512,
       compatible_brands: vec![b"mp41".into()],
     });
   },
   Moov::KIND => {
     // Manually decode the moov
     match header.size {
       Some(size) => { /* read size bytes */ },
       None => { /* read until EOF */ },
     };
   },
   _ => {
     // You can also use Any if you prefer
     let any = Any::read_atom(&header, &mut reader)?;
     println!("Unknown atom: {:?}", any);
   }
};
```

### Asynchronous IO
Enable using the `tokio` feature.
It's the same as the above two but using the `AsyncReadFrom`, `AsyncWriteTo`, and `AsyncReadAtom` traits instead.

There's also the `bytes` features which enables encoding for `Bytes` and `BytesMut` from the `bytes` crate, often used with tokio.