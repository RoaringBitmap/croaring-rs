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
