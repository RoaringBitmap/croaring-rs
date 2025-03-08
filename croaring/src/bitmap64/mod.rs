//! A compressed bitmap which can hold 64-bit integers

pub use self::iter::{Bitmap64Cursor, Bitmap64Iterator};
use core::marker::PhantomData;

mod imp;
mod iter;
mod ops;
mod serialization;
mod view;

pub use self::serialization::{Deserializer, Serializer};

/// A Bitmap which can hold 64-bit integers
pub struct Bitmap64 {
    raw: core::ptr::NonNull<ffi::roaring64_bitmap_t>,
}
unsafe impl Sync for Bitmap64 {}
unsafe impl Send for Bitmap64 {}

/// A frozen view of a bitmap, backed by a byte slice
///
/// All read-only methods for [`Bitmap64`] are also usable on a [`Bitmap64View`]
#[repr(transparent)]
pub struct Bitmap64View<'a> {
    // We must only expose a shared reference to the bitmap, to ensure it is not modified
    bitmap: Bitmap64,
    // Rust lifetime rules will ensure we don't outlive our data, or modify it behind the scenes
    phantom: PhantomData<&'a [u8]>,
}

unsafe impl<'a> Sync for Bitmap64View<'a> {}
unsafe impl<'a> Send for Bitmap64View<'a> {}

/// Detailed statistics on the composition of a bitmap
///
/// See [`Bitmap64::statistics`] for more information
pub type Statistics = ffi::roaring64_statistics_t;
