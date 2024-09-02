[![crates.io](https://img.shields.io/crates/v/mp4-atom)](https://crates.io/crates/mp4-atom)
[![docs.rs](https://img.shields.io/docsrs/mp4-atom)](https://docs.rs/mp4-atom)
[![discord](https://img.shields.io/discord/1124083992740761730)](https://discord.gg/FCYF3p99mr)

# mp4-atom
A library for decoding and encoding MP4 atoms.

This library provides encoding for the ISO Base Media File Format (ISO/IEC 14496-12).
It's meant to be low level, performing encoding/decoding of the binary format without
validation or interpretation of the data. You have to know what boxes to expect!

## Notice
This library is still in development and hasn't been tested on enough real MP4 files.
Please report any issues you find!
