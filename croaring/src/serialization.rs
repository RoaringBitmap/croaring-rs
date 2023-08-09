/// The `Portable` format is meant to be compatible with other roaring bitmap libraries, such as Go or Java.
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
pub enum Frozen {}

impl Frozen {
    /// The frozen format requires bitmaps are aligned to 32 bytes.
    pub const REQUIRED_ALIGNMENT: usize = 32;
}
