#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
//! Rust wrapper for `CRoaring` (a C/C++ implementation at <https://github.com/RoaringBitmap/CRoaring>)
//!
//! Provides Compressed Bitmaps, which act like a set of integers in an efficient way.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod bitmap;
pub mod bitmap64;
pub mod bitset;

#[cfg(feature = "alloc")]
pub mod treemap;

mod callback;
#[cfg(any(feature = "alloc", feature = "allocator-api2"))]
mod rust_alloc;
mod serialization;

mod sealed {
    pub trait Sealed {}
}

use core::mem::offset_of;
use core::ops::Bound;
pub use serialization::*;

pub use bitmap::{Bitmap, BitmapView};
pub use bitmap64::{Bitmap64, Bitmap64View};
pub use bitset::Bitset;

#[cfg(feature = "alloc")]
pub use treemap::Treemap;

#[cfg(feature = "allocator-api2")]
pub use rust_alloc::configure_custom_alloc;
#[cfg(feature = "alloc")]
pub use rust_alloc::configure_rust_alloc;

/// An inclusive range
///
/// `RangeInclusive<u32>` is guaranteed to be ABI compatible with `roaring_uint32_range_closed_t`,
/// `RangeInclusive<u64>` is guaranteed to be ABI compatible with `roaring64_range_closed_t`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
pub struct RangeInclusive<Idx> {
    /// The lower bound of the range (inclusive).
    pub start: Idx,
    /// The upper bound of the range (inclusive).
    pub last: Idx,
}
impl<Idx> From<core::ops::RangeInclusive<Idx>> for RangeInclusive<Idx> {
    fn from(value: core::ops::RangeInclusive<Idx>) -> Self {
        let (start, last) = value.into_inner();
        Self { start, last }
    }
}

impl<Idx> From<RangeInclusive<Idx>> for core::ops::RangeInclusive<Idx> {
    fn from(value: RangeInclusive<Idx>) -> Self {
        core::ops::RangeInclusive::new(value.start, value.last)
    }
}

impl<Idx> From<core::range::RangeInclusive<Idx>> for RangeInclusive<Idx> {
    fn from(value: core::range::RangeInclusive<Idx>) -> Self {
        let core::range::RangeInclusive { start, last } = value;
        Self { start, last }
    }
}

impl<Idx> From<RangeInclusive<Idx>> for core::range::RangeInclusive<Idx> {
    fn from(value: RangeInclusive<Idx>) -> Self {
        let RangeInclusive { start, last } = value;
        core::range::RangeInclusive { start, last }
    }
}

impl<Idx> core::ops::RangeBounds<Idx> for RangeInclusive<Idx> {
    fn start_bound(&self) -> Bound<&Idx> {
        Bound::Included(&self.start)
    }

    fn end_bound(&self) -> Bound<&Idx> {
        Bound::Included(&self.last)
    }
}

const _: () = {
    type FFIRange = ffi::roaring_uint32_range_closed_t;
    type RustRange = RangeInclusive<u32>;

    assert!(size_of::<FFIRange>() == size_of::<RustRange>());
    assert!(align_of::<FFIRange>() == align_of::<RustRange>());
    assert!(offset_of!(FFIRange, min) == offset_of!(RustRange, start));
    assert!(offset_of!(FFIRange, max) == offset_of!(RustRange, last));
};

const _: () = {
    type FFIRange = ffi::roaring64_range_closed_t;
    type RustRange = RangeInclusive<u64>;

    assert!(size_of::<FFIRange>() == size_of::<RustRange>());
    assert!(align_of::<FFIRange>() == align_of::<RustRange>());
    assert!(offset_of!(FFIRange, min) == offset_of!(RustRange, start));
    assert!(offset_of!(FFIRange, max) == offset_of!(RustRange, last));
};
