#![deny(missing_docs)]
//! Rust wrapper for `CRoaring` (a C/C++ implementation at <https://github.com/RoaringBitmap/CRoaring>)
//!
//! Provides Compressed Bitmaps, which act like a set of integers in an efficient way.

pub mod bitmap;
pub mod bitmap64;
pub mod bitset;
pub mod treemap;

mod callback;
mod serialization;

pub use serialization::*;

pub use bitmap::Bitmap;
pub use bitmap64::Bitmap64;
pub use bitset::Bitset;
pub use treemap::Treemap;

pub use bitmap::BitmapView;
