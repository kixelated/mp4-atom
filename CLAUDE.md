# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

`mp4-atom` is a Rust library for encoding/decoding ISO Base Media File Format (ISO/IEC 14496-12) atoms at a low level. MP4 files consist of atoms (boxes) containing data, identified by 4-byte FourCC codes (like `moov`, `mdat`, `trak`).

## Development Commands

Build and check the code:
```bash
cargo build                                    # Build the library
cargo build --all-features                     # Build with all features (bytes, tokio, serde)
cargo check --all-targets --all-features      # Check all targets with all features
```

Run tests:
```bash
cargo test                                     # Run all tests
cargo test --all-targets                      # Run tests on all targets
cargo test <test_name>                        # Run a specific test
```

Code quality and CI checks:
```bash
just check                                     # Run full CI checks (check, clippy, fmt, shear)
just test                                      # Run all tests
just fix                                       # Auto-fix issues (fix, clippy fix, fmt, shear fix)
```

Linting and formatting:
```bash
cargo clippy --all-targets --all-features -- -D warnings  # Run clippy with strict warnings
cargo fmt -- --check                                       # Check formatting
cargo fmt                                                  # Auto-format code
```

## Architecture

### Core Traits

The library revolves around several key traits defined in `src/`:

- **`Atom`** (`atom.rs`): Core trait for all atom types, provides `KIND` constant and encode/decode methods
- **`Decode`/`Encode`** (`coding.rs`): For working with byte slices
- **`ReadFrom`/`WriteTo`** (`io.rs`): Synchronous IO operations
- **`AsyncReadFrom`/`AsyncWriteTo`** (`tokio/`): Async IO with tokio feature
- **`Buf`/`BufMut`** (`buf.rs`): Custom buffer traits for contiguous byte slices

### Module Structure

Atoms are organized hierarchically matching the MP4 format:

- `src/moov/`: Movie metadata atoms (mvhd, trak, etc.)
  - `trak/`: Track-level atoms
    - `mdia/`: Media information
      - `minf/stbl/`: Sample table atoms
        - `stsd/`: Sample descriptions for codecs (h264, hevc, av01, mp4a, etc.)
- `src/moof/`: Movie fragment atoms for streaming
- `src/meta/`: Metadata atoms (HEIF/AVIF support)
- `src/`: Top-level atoms (ftyp, mdat, free, etc.)

### Key Design Patterns

1. **Atom Encoding/Decoding**: Each atom type implements the `Atom` trait with:
   - `KIND`: FourCC identifier
   - `decode_body()`: Parse atom contents
   - `encode_body()`: Write atom contents

2. **Header Handling**: Use `Header::read_from()` to handle large atoms without loading into memory

3. **Any Enum**: `src/any.rs` provides a giant enum of all supported atoms for flexible decoding

4. **Error Handling**: All operations return `Result<T>` with custom error types in `error.rs`

## Feature Flags

- `bytes`: Enables support for `bytes` crate types (Bytes, BytesMut)
- `tokio`: Enables async IO support
- `serde`: Enables serialization/deserialization support

## Testing

Tests are located in:
- Unit tests: Within each module using `#[cfg(test)]`
- Integration tests: `src/test/` directory with sample MP4 files for different codecs

Run specific codec tests:
```bash
cargo test h264    # Test H.264 support
cargo test hevc    # Test HEVC support
cargo test av1     # Test AV1 support
```