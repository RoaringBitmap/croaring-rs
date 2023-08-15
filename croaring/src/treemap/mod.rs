//! Treemap is a RoaringBitmap-based structure that supports 64bit unsigned
//! integer values. Implemented as a [`BTreeMap`].
//!
//! Java version can be found at <https://github.com/RoaringBitmap/RoaringBitmap/blob/master/roaringbitmap/src/main/java/org/roaringbitmap/longlong/Roaring64NavigableMap.java>
//! C++ version - <https://github.com/RoaringBitmap/CRoaring/blob/master/cpp/roaring64map.hh>
//!
//! # Example
//!
//! ```rust
//! use std::u32;
//! use croaring::Treemap;
//!
//! let mut treemap = Treemap::new();
//! treemap.add(3);
//! assert!(treemap.contains(3));
//! treemap.add(u32::MAX as u64);
//! assert!(treemap.contains(u32::MAX as u64));
//! treemap.add(u64::from(u32::MAX) + 1);
//! assert!(treemap.contains(u64::from(u32::MAX)+ 1));
//! assert_eq!(treemap.cardinality(), 3);
//! ```
use crate::Bitmap;
use std::collections::BTreeMap;

mod imp;
mod iter;
mod ops;
mod serialization;
mod util;

pub use iter::TreemapIterator;
pub use serialization::{Deserializer, Serializer};

/// A RoaringBitmap-based structure that supports 64bit unsigned integer values
///
/// Implemented as a [`BTreeMap`] of [`Bitmap`]s.
#[derive(Clone, PartialEq, Eq)]
pub struct Treemap {
    /// The underlying map of bitmaps
    pub map: BTreeMap<u32, Bitmap>,
}
