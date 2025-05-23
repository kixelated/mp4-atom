# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.1](https://github.com/kixelated/mp4-atom/compare/v0.8.0...v0.8.1) - 2025-05-15

### Other

- Fix some missing serde attributes. ([#42](https://github.com/kixelated/mp4-atom/pull/42))

## [0.8.0](https://github.com/kixelated/mp4-atom/compare/v0.7.2...v0.8.0) - 2025-05-13

### Added

- add uncompressed video support ([#38](https://github.com/kixelated/mp4-atom/pull/38))
- add TAIClockInfoBox (taic) support ([#37](https://github.com/kixelated/mp4-atom/pull/37))
- add support for BitRateBox (btrt) ([#35](https://github.com/kixelated/mp4-atom/pull/35))
- add ColourInformationBox (colr) ([#34](https://github.com/kixelated/mp4-atom/pull/34))
- add PixelAspectRatioBox (pasp) ([#32](https://github.com/kixelated/mp4-atom/pull/32))

### Fixed

- correct hvcC parsing and encoding ([#41](https://github.com/kixelated/mp4-atom/pull/41))

### Other

- Add HEIF support (top level metabox) ([#40](https://github.com/kixelated/mp4-atom/pull/40))
- Initial Opus support. ([#39](https://github.com/kixelated/mp4-atom/pull/39))
- Add auxiliary information box support ([#36](https://github.com/kixelated/mp4-atom/pull/36))

## [0.7.2](https://github.com/kixelated/mp4-atom/compare/v0.7.1...v0.7.2) - 2025-03-18

### Other

- Exhaustive ([#29](https://github.com/kixelated/mp4-atom/pull/29))
- Always parse the extended avcC atom. ([#30](https://github.com/kixelated/mp4-atom/pull/30))

## [0.7.1](https://github.com/kixelated/mp4-atom/compare/v0.7.0...v0.7.1) - 2025-03-09

### Other

- Opps. ([#27](https://github.com/kixelated/mp4-atom/pull/27))

## [0.7.0](https://github.com/kixelated/mp4-atom/compare/v0.6.0...v0.7.0) - 2025-03-09

### Other

- Add #[non_exhaustive] to the huge enums. ([#26](https://github.com/kixelated/mp4-atom/pull/26))
- Maybe support VP8 too? ([#25](https://github.com/kixelated/mp4-atom/pull/25))
- Added support for Hvc1 atoms. ([#23](https://github.com/kixelated/mp4-atom/pull/23))

## [0.6.0](https://github.com/kixelated/mp4-atom/compare/v0.5.0...v0.6.0) - 2025-03-03

### Other

- Fix vp9 support. ([#22](https://github.com/kixelated/mp4-atom/pull/22))
- Fix HEVC (h265) support. ([#20](https://github.com/kixelated/mp4-atom/pull/20))

## [0.5.0](https://github.com/kixelated/mp4-atom/compare/v0.4.0...v0.5.0) - 2025-03-03

### Other

- Initial AV1 support ([#17](https://github.com/kixelated/mp4-atom/pull/17))
- Use just for checks ([#18](https://github.com/kixelated/mp4-atom/pull/18))

## [0.4.0](https://github.com/kixelated/mp4-atom/compare/v0.3.0...v0.4.0) - 2025-01-14

### Other

- Fix avcC parsing. ([#16](https://github.com/kixelated/mp4-atom/pull/16))
- Add fuzzing and fix some bugs. ([#14](https://github.com/kixelated/mp4-atom/pull/14))

## [0.3.0](https://github.com/kixelated/mp4-atom/compare/v0.2.1...v0.3.0) - 2024-10-18

### Other

- Add decode_maybe ([#12](https://github.com/kixelated/mp4-atom/pull/12))
- Fix some user-provided MP4 files. ([#11](https://github.com/kixelated/mp4-atom/pull/11))
- Zero copy ([#9](https://github.com/kixelated/mp4-atom/pull/9))

## [0.2.1](https://github.com/kixelated/mp4-atom/compare/v0.2.0...v0.2.1) - 2024-09-24

### Other

- Crude serde support. ([#6](https://github.com/kixelated/mp4-atom/pull/6))
- Fix trun first_sample ([#7](https://github.com/kixelated/mp4-atom/pull/7))
- Added read_until helper. ([#5](https://github.com/kixelated/mp4-atom/pull/5))
- Fix a README link

## [0.2.0](https://github.com/kixelated/mp4-atom/compare/v0.1.0...v0.2.0) - 2024-09-06

### Other
- Async support and better documentation.
- v0.1.0
