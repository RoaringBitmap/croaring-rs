//! A compressed bitmap which can hold 64-bit integers

pub use self::iter::{Bitmap64Cursor, Bitmap64Iterator};

mod imp;
mod iter;
mod ops;
mod serialization;

pub use self::serialization::{Deserializer, Serializer};

/// A Bitmap which can hold 64-bit integers
pub struct Bitmap64 {
    raw: std::ptr::NonNull<ffi::roaring64_bitmap_t>,
}
unsafe impl Sync for Bitmap64 {}
unsafe impl Send for Bitmap64 {}
