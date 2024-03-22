//! Rust wrapper for `CRoaring` (a C/C++ implementation at <https://github.com/RoaringBitmap/CRoaring>)
//!
//! The original Java version can be found at <https://github.com/RoaringBitmap/RoaringBitmap>
//! # Example
//!
//! ```rust
//! use croaring::Bitmap;
//!
//! let mut rb1 = Bitmap::new();
//! rb1.add(1);
//! rb1.add(2);
//! rb1.add(3);
//! rb1.add(4);
//! rb1.add(5);
//! rb1.add(100);
//! rb1.add(1000);
//! rb1.run_optimize();
//!
//! let mut rb2 = Bitmap::new();
//! rb2.add(3);
//! rb2.add(4);
//! rb2.add(1000);
//! rb2.run_optimize();
//!
//! let mut rb3 = Bitmap::new();
//!
//! assert_eq!(rb1.cardinality(), 7);
//! assert!(rb1.contains(3));
//!
//! rb1.and_inplace(&rb2);
//! rb3.add(5);
//! rb3.or_inplace(&rb1);
//!
//! let mut rb4 = Bitmap::fast_or(&[&rb1, &rb2, &rb3]);
//!
//! rb1.and_inplace(&rb2);
//! println!("{:?}", rb1);
//!
//! rb3.add(5);
//! rb3.or_inplace(&rb1);
//!
//! println!("{:?}", rb1);
//!
//! rb3.add(5);
//! rb3.or_inplace(&rb1);
//!
//! println!("{:?}", rb3.to_vec());
//! println!("{:?}", rb3);
//! println!("{:?}", rb4);
//!
//! rb4 = Bitmap::fast_or(&[&rb1, &rb2, &rb3]);
//!
//! println!("{:?}", rb4);
//! ```

use std::marker::PhantomData;

/// A compressed bitmap
// Must be repr(transparent) and match BitmapView, to allow safe transmute between
// &BitmapView and &Bitmap
#[repr(transparent)]
pub struct Bitmap {
    bitmap: ffi::roaring_bitmap_t,
}

unsafe impl Sync for Bitmap {}
unsafe impl Send for Bitmap {}

/// A frozen view of a bitmap, backed by a byte slice
///
/// All read-only methods for [`Bitmap`] are also usable on a [`BitmapView`]
#[repr(transparent)]
pub struct BitmapView<'a> {
    bitmap: ffi::roaring_bitmap_t,
    // Rust lifetime rules will ensure we don't outlive our data, or modify it behind the scenes
    phantom: PhantomData<&'a [u8]>,
}

unsafe impl<'a> Sync for BitmapView<'a> {}
unsafe impl<'a> Send for BitmapView<'a> {}

/// Detailed statistics on the composition of a bitmap
pub type Statistics = ffi::roaring_statistics_s;

mod imp;
mod iter;
mod lazy;
mod ops;
mod serialization;
mod view;

pub use self::iter::{BitmapCursor, BitmapIterator};
pub use self::lazy::LazyBitmap;
pub use self::serialization::{Deserializer, Serializer, ViewDeserializer};
