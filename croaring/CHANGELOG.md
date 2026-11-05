# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.9.1](https://github.com/RoaringBitmap/croaring-rs/compare/croaring-v2.9.0...croaring-v2.9.1) - 2026-11-05

### Other
- Update to croaring-sys 5.5.5 (by @github-actions[bot]) - #263
- Update to croaring-sys 5.5.3 (by @github-actions[bot]) - #261

## [2.9.0](https://github.com/RoaringBitmap/croaring-rs/compare/croaring-v2.8.0...croaring-v2.9.0) - 2026-11-03

### Added
- [**breaking**] Require configure_custom_alloc allocators to be Sync (by @Dr-Emann) - #259
  - This is actually a breaking change, but it is a soundness fix.
- Add BitmapView::try_deserialize (by @Dr-Emann) - #259
- Add Bitmap::word_range_to_bitset and word_range_to_bitset_uninit (by @Dr-Emann) - #254

### Fixed
- Handle counts above u32::MAX in BitmapIterator::nth (by @Dr-Emann) - #259
- Repair bitmap after lazy_batch even if the closure panics (by @Dr-Emann) - #259
- Reject oversized sizes in global allocator realloc (by @Dr-Emann) - #258
- Correct Bitset self-comparison for empty bitsets (by @Dr-Emann) - #258
- Keep BitsetIterator size_hint from underflowing after exhaustion (by @Dr-Emann) - #258
- Handle u64::MAX and zero step in Bitmap64::from_range_with_step (by @Dr-Emann) - #258
- Include u32::MAX in unbounded Bitmap ranges (by @Dr-Emann) - #258
- Pass allocation start to custom allocator in realloc (by @Dr-Emann) - #258

### Other
- Use associated numeric constants in tests instead of std::u32 (by @Dr-Emann) - #259
- Fix unclear and truncated Bitmap and Bitmap64 documentation (by @Dr-Emann) - #259
- Fix typos and close unclosed code blocks (by @Dr-Emann) - #259
- Make is_strict_subset and fast_or examples test what they document (by @Dr-Emann) - #259
- Correct inaccurate descriptions of Bitmap and Bitmap64 methods (by @Dr-Emann) - #259
- Deprecate Bitmap and Bitmap64 deserialize in favor of try_deserialize (by @Dr-Emann) - #259
- Note that Bitmap64 frozen data needs 64 byte alignment (by @Dr-Emann) - #259
- Assert Send and Sync for public types with manual impls (by @Dr-Emann) - #259
- Call init() in the unbounded range tests (by @Dr-Emann) - #259
- Document concurrency and mixing requirements of configure_* allocator hooks (by @Dr-Emann) - #259
- Portable frozen view no longer requires aligned data (by @Dr-Emann) - #254
- *(deps)* Bump roaring from 0.11.5 to 0.12.0 (by @dependabot[bot]) - #249
- Update to croaring-sys 5.4.0 (by @github-actions[bot]) - #253
- Update to croaring-sys 5.2.3 (by @Dr-Emann) - #246

## [2.8.0](https://github.com/RoaringBitmap/croaring-rs/compare/croaring-v2.7.0...croaring-v2.8.0) - 2026-09-20

### Added
- Add the ability to deserialize a portable 64 bit without copies (by @Dr-Emann) - #242

### Other
- Optimize 64-bit iterator access (by @Dr-Emann) - #243
- *(deps)* Update croaring to work with croaring-sys (by @Dr-Emann) - #242
- Update to croaring 4.6.2 (by @Dr-Emann) - #237
- Use new roaring64_bitmap_contains_range_closed function for intersect_with_range (by @Dr-Emann) - #234
- *(croaring-sys)* Update to CRoaring 4.7.1 (by @Dr-Emann) - #233

## [2.7.0](https://github.com/RoaringBitmap/croaring-rs/compare/croaring-v2.6.0...croaring-v2.7.0) - 2026-06-25

### Added
- Add reverse and range bulk-read cursor methods (by @Dr-Emann) - #230

### Other
- Switch croaring to edition 2024 (by @Dr-Emann) - #231
- *(croaring-sys)* Update to CRoaring 4.7.0 (by @Dr-Emann) - #226

## [2.6.0](https://github.com/RoaringBitmap/croaring-rs/compare/croaring-v2.5.2...croaring-v2.6.0) - 2026-04-02

### Added
- Remove_run_compression, add_offset, Clone::clone_from (by @Dr-Emann) - #220

### Other
- Update to croaring 4.6.1 (by @Dr-Emann) - #218
- Test deserialing at offsets (by @Dr-Emann) - #219

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
