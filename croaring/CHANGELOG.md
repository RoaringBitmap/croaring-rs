# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.5.2](https://github.com/RoaringBitmap/croaring-rs/compare/croaring-v2.5.1...croaring-v2.5.2) - 2026-01-02

### Other
- Update to croaring 4.5.1 (by @Dr-Emann) - #212
- *(deps)* Bump allocator-api2 from 0.3.1 to 0.4.0 (by @dependabot[bot]) - #209
- *(deps)* Bump criterion from 0.7.0 to 0.8.1 (by @dependabot[bot]) - #211
- Update CI badge in README.md (by @lemire)
- Update croaring to 4.4.1 (by @Dr-Emann) - #202

## [2.5.1](https://github.com/RoaringBitmap/croaring-rs/compare/croaring-v2.5.0...croaring-v2.5.1) - 2025-10-02

### Other
- Fix docs.rs builds

## [2.5.0](https://github.com/RoaringBitmap/croaring-rs/compare/croaring-v2.4.0...croaring-v2.5.0) - 2025-10-02

### Added

- Add skip functions to BitmapCursor, and implement BitmapIter::nth (by @Dr-Emann) - #199

### Other

- Update croaring to 4.4.0 (by @Dr-Emann) - #198

## [2.4.0](https://github.com/RoaringBitmap/croaring-rs/compare/croaring-v2.3.1...croaring-v2.4.0) - 2025-09-04

### Added

- Introduce configure_custom_alloc (by @Dr-Emann) - #194

### Fixed

- Appease clippy, explicitly indicate elided lifetime (by @Dr-Emann) - #191

### Other

- Update croaring to 4.3.10 (by @Dr-Emann) - #191
- Update dependencies (by @Dr-Emann) - #191
- Update croaring to 4.3.7 (by @Dr-Emann) - #191
- *(deps)* Bump criterion from 0.5.1 to 0.6.0 (by @dependabot[bot]) - #182

## [2.3.1](https://github.com/RoaringBitmap/croaring-rs/compare/v2.3.0...v2.3.1) - 2025-06-01

### Other

- Correct return docs for run_optimize function (by @Dr-Emann) - #177

## [2.3.0](https://github.com/RoaringBitmap/croaring-rs/compare/croaring-v2.2.0...croaring-v2.3.0) - 2025-03-25

### Added

- Frozen serialization format for bitmap64 (by @Dr-Emann) - #172
- Enable configuring rust global alloc as croaring allocator (by @Dr-Emann) - #171

## [2.2.0](https://github.com/RoaringBitmap/croaring-rs/compare/croaring-v2.1.1...croaring-v2.2.0) - 2024-12-17

### Added

- Bitset::is_empty

### Fixed

- Correct doc test for Bitset::minimum to match new behavior

### Other

- Loosen compile asserts to only limit roaring major version
