//! Dense bitset implementation

mod imp;
mod iter;
mod ops;

/// A dense bitset
pub struct Bitset {
    bitset: ffi::bitset_t,
}

pub use self::iter::BitsetIterator;

unsafe impl Sync for Bitset {}
unsafe impl Send for Bitset {}
