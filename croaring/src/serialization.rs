/// The `Portable` format is meant to be compatible with other roaring bitmap libraries, such as Go or Java.
///
/// Note despite the name, it is not fully portable: it depends on native endianness.
///
/// It's defined here: <https://github.com/RoaringBitmap/RoaringFormatSpec>
pub enum Portable {}

/// The `Native` format format can sometimes be more space efficient than [`Portable`],
///
/// e.g. when the data is sparse. It's not compatible with Java and Go implementations.
/// Use [`Portable`] for that purpose.
pub enum Native {}

/// The `Frozen` format imitates memory layout of the underlying C library.
///
/// This reduces amount of allocations and copying required during deserialization, though
/// `Portable` offers comparable performance.
///
/// Note that because frozen serialization format imitates C memory layout
/// of `roaring_bitmap_t`, it is not fixed. It is different on big/little endian
/// platforms and can be changed in future.
pub enum Frozen {}

impl Frozen {
    /// The frozen format requires bitmaps are aligned to 32 bytes.
    pub const REQUIRED_ALIGNMENT: usize = 32;
}

mod private {
    use crate::{Native, Portable};

    #[allow(unused)]
    pub trait NoAlign: crate::sealed::Sealed {}
    impl NoAlign for Native {}
    impl NoAlign for Portable {}
}

#[allow(unused)]
pub(crate) use private::NoAlign;

/// The `JvmLegacy` format is meant to be compatible with the original Java implementation of `Roaring64NavigableMap`
///
/// It is used only for [Treemap][crate::Treemap]s, not bitmaps.
///
/// See <https://github.com/RoaringBitmap/RoaringBitmap/blob/2669c4f5a49ee7da5ff4cd70e18ee5520018d6a5/RoaringBitmap/src/main/java/org/roaringbitmap/longlong/Roaring64NavigableMap.java#L1215-L1238>
pub enum JvmLegacy {}

#[cfg(feature = "alloc")]
pub(crate) fn get_aligned_spare_capacity(
    dst: &mut alloc::vec::Vec<u8>,
    align: usize,
    required_len: usize,
) -> &mut [core::mem::MaybeUninit<u8>] {
    let max_padding = align - 1;
    let extra_align_required =
        |v: &mut alloc::vec::Vec<u8>| v.spare_capacity_mut().as_ptr().align_offset(align);
    let mut extra_offset = extra_align_required(dst);
    if dst.spare_capacity_mut().len() < required_len + extra_offset {
        dst.reserve(required_len.checked_add(max_padding).unwrap());
        // Need to recompute offset after reserve, as the buffer may have been reallocated and
        // the end of the buffer may be somewhere else
        extra_offset = extra_align_required(dst);
    }
    let mut data_start = dst.len();
    if extra_offset != 0 {
        data_start = data_start.checked_add(extra_offset).unwrap();
        // we must initialize up to offset
        dst.resize(data_start, 0);
    }
    debug_assert_eq!(dst.len(), data_start);
    let spare_capacity = dst.spare_capacity_mut();
    debug_assert!(spare_capacity.len() >= required_len);
    debug_assert_eq!(spare_capacity.as_ptr().align_offset(align), 0);

    &mut spare_capacity[..required_len]
}
