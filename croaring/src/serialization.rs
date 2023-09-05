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

    // The most padding required to get 32 byte alignment is 31 bytes.
    pub(crate) const MAX_PADDING: usize = Self::REQUIRED_ALIGNMENT - 1;

    #[inline]
    pub(crate) const fn required_padding(x: usize) -> usize {
        match x % Self::REQUIRED_ALIGNMENT {
            0 => 0,
            r => Self::REQUIRED_ALIGNMENT - r,
        }
    }
}

/// The `JvmLegacy` format is meant to be compatible with the original Java implementation of `Roaring64NavigableMap`
///
/// It is used only for [Treemap][crate::Treemap]s, not bitmaps.
///
/// See <https://github.com/RoaringBitmap/RoaringBitmap/blob/2669c4f5a49ee7da5ff4cd70e18ee5520018d6a5/RoaringBitmap/src/main/java/org/roaringbitmap/longlong/Roaring64NavigableMap.java#L1215-L1238>
pub enum JvmLegacy {}
