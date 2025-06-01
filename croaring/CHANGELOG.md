# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.3.1](https://github.com/RoaringBitmap/croaring-rs/compare/croaring-v2.3.0...croaring-v2.3.1) - 2025-06-01

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
