use crate::Bitset;
use ffi::roaring_bitmap_t;
use std::convert::TryInto;
use std::ffi::{c_void, CStr};
use std::ops::{Bound, RangeBounds};
use std::{mem, ptr};

use super::serialization::{Deserializer, Serializer};
use super::{Bitmap, Statistics};

impl Bitmap {
    #[inline]
    #[allow(clippy::assertions_on_constants)]
    pub(crate) unsafe fn take_heap(p: *mut roaring_bitmap_t) -> Self {
        // Based heavily on the `roaring.hh` cpp header from croaring

        assert!(!p.is_null());
        let result = Self { bitmap: *p };
        // This depends somewhat heavily on the implementation of croaring,
        // In particular, that `roaring_bitmap_t` doesn't store any pointers into itself
        // (it can be moved safely), and can be freed with `free`, without freeing the underlying
        // containers and auxiliary data. Ensure this is still valid every time we update
        // the version of croaring.
        const _: () = assert!(ffi::ROARING_VERSION_MAJOR == 3 && ffi::ROARING_VERSION_MINOR == 0);
        ffi::roaring_free(p.cast::<c_void>());
        result
    }

    /// Creates a new bitmap (initially empty)
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap = Bitmap::new();
    ///
    /// assert!(bitmap.is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::with_container_capacity(0)
    }

    /// Creates a new bitmap (initially empty) with a provided
    /// container-storage capacity (it is a performance hint).
    ///
    /// Note that this is in units of containers, not values: each container holds up to
    /// 2^16 values.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap = Bitmap::with_container_capacity(1_000);
    ///
    /// assert!(bitmap.is_empty());
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_init_with_capacity")]
    #[must_use]
    pub fn with_container_capacity(capacity: u32) -> Self {
        let mut bitmap = mem::MaybeUninit::uninit();
        let success =
            unsafe { ffi::roaring_bitmap_init_with_capacity(bitmap.as_mut_ptr(), capacity) };
        assert!(success);

        Bitmap {
            bitmap: unsafe { bitmap.assume_init() },
        }
    }

    /// Add the integer element to the bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::new();
    /// bitmap.add_many(&[1, 2, 3]);
    ///
    /// assert!(!bitmap.is_empty());
    /// assert!(bitmap.contains(1));
    /// assert!(bitmap.contains(2));
    /// assert!(bitmap.contains(3));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_add_many")]
    pub fn add_many(&mut self, elements: &[u32]) {
        unsafe { ffi::roaring_bitmap_add_many(&mut self.bitmap, elements.len(), elements.as_ptr()) }
    }

    /// Add the integer element to the bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::new();
    /// assert!(bitmap.is_empty());
    /// bitmap.add(1);
    /// assert!(!bitmap.is_empty());
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_add")]
    pub fn add(&mut self, element: u32) {
        unsafe { ffi::roaring_bitmap_add(&mut self.bitmap, element) }
    }

    /// Add the integer element to the bitmap. Returns true if the value was
    /// added, false if the value was already in the bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::new();
    /// assert!(bitmap.add_checked(1));
    /// assert!(!bitmap.add_checked(1));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_add_checked")]
    pub fn add_checked(&mut self, element: u32) -> bool {
        unsafe { ffi::roaring_bitmap_add_checked(&mut self.bitmap, element) }
    }

    /// Add all values in range
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::new();
    /// bitmap1.add_range((1..3));
    ///
    /// assert!(!bitmap1.is_empty());
    /// assert!(bitmap1.contains(1));
    /// assert!(bitmap1.contains(2));
    /// assert!(!bitmap1.contains(3));
    ///
    /// let mut bitmap2 = Bitmap::new();
    /// bitmap2.add_range((3..1));
    /// assert!(bitmap2.is_empty());
    ///
    /// let mut bitmap3 = Bitmap::new();
    /// bitmap3.add_range((3..3));
    /// assert!(bitmap3.is_empty());
    ///
    /// let mut bitmap4 = Bitmap::new();
    /// bitmap4.add_range(..=2);
    /// bitmap4.add_range(u32::MAX..=u32::MAX);
    /// assert!(bitmap4.contains(0));
    /// assert!(bitmap4.contains(1));
    /// assert!(bitmap4.contains(2));
    /// assert!(bitmap4.contains(u32::MAX));
    /// assert_eq!(bitmap4.cardinality(), 4);
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_add_range_closed")]
    pub fn add_range<R: RangeBounds<u32>>(&mut self, range: R) {
        let (start, end) = range_to_inclusive(range);
        unsafe {
            ffi::roaring_bitmap_add_range_closed(&mut self.bitmap, start, end);
        }
    }

    /// Remove all values in range
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::new();
    /// bitmap.add_range((1..4));
    /// assert!(!bitmap.is_empty());
    ///
    /// bitmap.remove_range((1..3));
    ///
    /// assert!(!bitmap.contains(1));
    /// assert!(!bitmap.contains(2));
    /// assert!(bitmap.contains(3));
    ///
    /// bitmap.add_range(u32::MAX..=u32::MAX);
    /// assert!(bitmap.contains(u32::MAX));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_remove_range_closed")]
    pub fn remove_range<R: RangeBounds<u32>>(&mut self, range: R) {
        let (start, end) = range_to_inclusive(range);
        unsafe {
            ffi::roaring_bitmap_remove_range_closed(&mut self.bitmap, start, end);
        }
    }

    /// Check whether a range of values of range are present
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap = Bitmap::of(&[1, 2]);
    /// assert!(bitmap.contains_range((1..3)));
    ///
    /// let mut bitmap = bitmap.clone();
    /// bitmap.add(u32::MAX - 1);
    /// bitmap.add(u32::MAX);
    /// assert!(bitmap.contains_range((u32::MAX - 1)..=u32::MAX))
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_contains_range")]
    pub fn contains_range<R: RangeBounds<u32>>(&self, range: R) -> bool {
        let (start, end) = range_to_exclusive(range);
        unsafe { ffi::roaring_bitmap_contains_range(&self.bitmap, start, end) }
    }

    /// Empties the bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::new();
    /// bitmap.add(1);
    /// bitmap.add(2);
    /// bitmap.clear();
    ///
    /// assert!(bitmap.is_empty());
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_clear")]
    pub fn clear(&mut self) {
        unsafe { ffi::roaring_bitmap_clear(&mut self.bitmap) }
    }

    /// Clear the integer element from the bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::new();
    /// bitmap.add(1);
    /// bitmap.remove(1);
    ///
    /// assert!(bitmap.is_empty());
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_remove")]
    pub fn remove(&mut self, element: u32) {
        unsafe { ffi::roaring_bitmap_remove(&mut self.bitmap, element) }
    }

    /// Remove the integer element from the bitmap. Returns true if a the value
    /// was removed, false if the value was present in the bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::new();
    /// bitmap.add(1);
    /// assert!(bitmap.remove_checked(1));
    /// assert!(!bitmap.remove_checked(1));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_remove_checked")]
    pub fn remove_checked(&mut self, element: u32) -> bool {
        unsafe { ffi::roaring_bitmap_remove_checked(&mut self.bitmap, element) }
    }

    /// Remove many values from the bitmap
    ///
    /// This should be faster than calling `remove` multiple times.
    ///
    /// In order to exploit this optimization, the caller should attempt to keep values with the same high 48 bits of
    /// the value as consecutive elements in `vals`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    /// let mut bitmap = Bitmap::of(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    /// bitmap.remove_many(&[1, 2, 3, 4, 5, 6, 7, 8]);
    /// assert_eq!(bitmap.to_vec(), vec![9]);
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_remove_many")]
    pub fn remove_many(&mut self, elements: &[u32]) {
        unsafe {
            ffi::roaring_bitmap_remove_many(&mut self.bitmap, elements.len(), elements.as_ptr())
        }
    }

    /// Contains returns true if the integer element is contained in the bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap = Bitmap::of(&[1]);
    ///
    /// assert!(bitmap.contains(1));
    /// assert!(!bitmap.contains(2));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_contains")]
    #[must_use]
    pub fn contains(&self, element: u32) -> bool {
        unsafe { ffi::roaring_bitmap_contains(&self.bitmap, element) }
    }

    /// Compute a new bitmap, which contains all values from this bitmap, but shifted by `offset`
    ///
    /// Any values which would be `< 0`, or `> u32::MAX` are dropped.
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap1 = Bitmap::of(&[0, 1, 1000, u32::MAX]);
    /// let shifted_down = bitmap1.add_offset(-1);
    /// assert_eq!(shifted_down.to_vec(), [0, 999, u32::MAX - 1]);
    /// let shifted_up = bitmap1.add_offset(1);
    /// assert_eq!(shifted_up.to_vec(), [1, 2, 1001]);
    /// let big_shifted = bitmap1.add_offset(i64::from(u32::MAX) + 1);
    /// assert_eq!(big_shifted.to_vec(), []);
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_add_offset")]
    #[must_use]
    pub fn add_offset(&self, offset: i64) -> Self {
        unsafe { Bitmap::take_heap(ffi::roaring_bitmap_add_offset(&self.bitmap, offset)) }
    }

    /// Returns number of elements in range
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap = Bitmap::of(&[1, 3, 4]);
    ///
    /// assert_eq!(bitmap.range_cardinality((..1)), 0);
    /// assert_eq!(bitmap.range_cardinality((..2)), 1);
    /// assert_eq!(bitmap.range_cardinality((2..5)), 2);
    /// assert_eq!(bitmap.range_cardinality((..5)), 3);
    /// assert_eq!(bitmap.range_cardinality((1..=4)), 3);
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_range_cardinality")]
    pub fn range_cardinality<R: RangeBounds<u32>>(&self, range: R) -> u64 {
        let (start, end) = range_to_exclusive(range);
        unsafe { ffi::roaring_bitmap_range_cardinality(&self.bitmap, start, end) }
    }

    /// Returns the number of integers contained in the bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap = Bitmap::of(&[1]);
    ///
    /// assert_eq!(bitmap.cardinality(), 1);
    ///
    /// let mut bitmap = bitmap.clone();
    ///
    /// bitmap.add(2);
    ///
    /// assert_eq!(bitmap.cardinality(), 2);
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_get_cardinality")]
    #[must_use]
    pub fn cardinality(&self) -> u64 {
        unsafe { ffi::roaring_bitmap_get_cardinality(&self.bitmap) }
    }

    /// And computes the intersection between two bitmaps and returns the result
    /// as a new bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap1 = Bitmap::of(&[1]);
    /// let bitmap2 = Bitmap::of(&[1, 2]);
    ///
    /// let bitmap3 = bitmap1.and(&bitmap2);
    ///
    /// assert!(bitmap3.contains(1));
    /// assert!(!bitmap3.contains(2));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_and")]
    #[must_use]
    pub fn and(&self, other: &Self) -> Self {
        unsafe { Self::take_heap(ffi::roaring_bitmap_and(&self.bitmap, &other.bitmap)) }
    }

    /// Computes the intersection between two bitmaps and stores the result
    /// in the current bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::of(&[15]);
    /// let bitmap2 = Bitmap::of(&[25]);
    /// let mut bitmap3 = Bitmap::of(&[15]);
    /// let bitmap4 = Bitmap::of(&[15, 25]);
    ///
    /// bitmap1.and_inplace(&bitmap2);
    ///
    /// assert_eq!(bitmap1.cardinality(), 0);
    /// assert!(!bitmap1.contains(15));
    /// assert!(!bitmap1.contains(25));
    ///
    /// bitmap3.and_inplace(&bitmap4);
    ///
    /// assert_eq!(bitmap3.cardinality(), 1);
    /// assert!(bitmap3.contains(15));
    /// assert!(!bitmap3.contains(25));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_and_inplace")]
    pub fn and_inplace(&mut self, other: &Self) {
        unsafe { ffi::roaring_bitmap_and_inplace(&mut self.bitmap, &other.bitmap) }
    }

    /// Or computes the union between two bitmaps and returns the result
    /// as a new bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap1 = Bitmap::of(&[15]);
    /// let bitmap2 = Bitmap::of(&[25]);
    ///
    /// let bitmap3 = bitmap1.or(&bitmap2);
    ///
    /// assert_eq!(bitmap3.cardinality(), 2);
    /// assert!(bitmap3.contains(15));
    /// assert!(bitmap3.contains(25));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_or")]
    #[must_use]
    pub fn or(&self, other: &Self) -> Self {
        unsafe { Self::take_heap(ffi::roaring_bitmap_or(&self.bitmap, &other.bitmap)) }
    }

    /// Computes the union between two bitmaps and stores the result in
    /// the current bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::of(&[15]);
    /// let bitmap2 = Bitmap::of(&[25]);
    ///
    /// bitmap1.or_inplace(&bitmap2);
    ///
    /// assert_eq!(bitmap1.cardinality(), 2);
    /// assert!(bitmap1.contains(15));
    /// assert!(bitmap1.contains(25));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_or_inplace")]
    pub fn or_inplace(&mut self, other: &Self) {
        unsafe { ffi::roaring_bitmap_or_inplace(&mut self.bitmap, &other.bitmap) }
    }

    /// Computes the union between many bitmaps quickly, as opposed to having
    /// to call or() repeatedly. Returns the result as a new bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap1 = Bitmap::of(&[15]);
    /// let bitmap2 = Bitmap::of(&[25]);
    /// let bitmap3 = Bitmap::of(&[35]);
    ///
    /// let bitmap4 = Bitmap::fast_or(&[&bitmap1, &bitmap2, &bitmap3]);
    ///
    /// assert_eq!(bitmap4.cardinality(), 3);
    /// assert!(bitmap4.contains(15));
    /// assert!(bitmap4.contains(25));
    /// assert!(bitmap4.contains(25));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_or_many")]
    #[must_use]
    pub fn fast_or(bitmaps: &[&Bitmap]) -> Self {
        let mut bms: Vec<*const ffi::roaring_bitmap_s> = bitmaps
            .iter()
            .map(|item| ptr::addr_of!(item.bitmap))
            .collect();

        unsafe { Self::take_heap(ffi::roaring_bitmap_or_many(bms.len(), bms.as_mut_ptr())) }
    }

    /// Compute the union of 'number' bitmaps using a heap. This can
    /// sometimes be faster than Bitmap::fast_or.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap1 = Bitmap::of(&[15]);
    /// let bitmap2 = Bitmap::of(&[25]);
    /// let bitmap3 = Bitmap::of(&[35]);
    ///
    /// let bitmap4 = Bitmap::fast_or_heap(&[&bitmap1, &bitmap2, &bitmap3]);
    ///
    /// assert_eq!(bitmap4.cardinality(), 3);
    /// assert!(bitmap4.contains(15));
    /// assert!(bitmap4.contains(25));
    /// assert!(bitmap4.contains(25));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_or_many_heap")]
    #[must_use]
    pub fn fast_or_heap(bitmaps: &[&Bitmap]) -> Self {
        let mut bms: Vec<*const ffi::roaring_bitmap_s> = bitmaps
            .iter()
            .map(|item| ptr::addr_of!(item.bitmap))
            .collect();

        let count = u32::try_from(bms.len()).expect("can only or up to 2^32 bitmaps");

        unsafe { Self::take_heap(ffi::roaring_bitmap_or_many_heap(count, bms.as_mut_ptr())) }
    }

    /// Computes the symmetric difference (xor) between two bitmaps
    /// and returns new bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap1 = Bitmap::of(&[15, 25]);
    /// let bitmap2 = Bitmap::of(&[25, 35]);
    ///
    /// let bitmap3 = bitmap1.xor(&bitmap2);
    ///
    /// assert_eq!(bitmap3.cardinality(), 2);
    /// assert!(bitmap3.contains(15));
    /// assert!(!bitmap3.contains(25));
    /// assert!(bitmap3.contains(35));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_xor")]
    #[must_use]
    pub fn xor(&self, other: &Self) -> Self {
        unsafe { Self::take_heap(ffi::roaring_bitmap_xor(&self.bitmap, &other.bitmap)) }
    }

    /// Inplace version of roaring_bitmap_xor, stores result in current bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::of(&[15, 25]);
    /// let bitmap2 = Bitmap::of(&[25, 35]);
    ///
    /// bitmap1.xor_inplace(&bitmap2);
    ///
    /// assert_eq!(bitmap1.cardinality(), 2);
    /// assert!(bitmap1.contains(15));
    /// assert!(!bitmap1.contains(25));
    /// assert!(bitmap1.contains(35));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_xor_inplace")]
    pub fn xor_inplace(&mut self, other: &Self) {
        unsafe { ffi::roaring_bitmap_xor_inplace(&mut self.bitmap, &other.bitmap) }
    }

    /// Computes the symmetric difference (xor) between multiple bitmaps
    /// and returns new bitmap as a result.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap1 = Bitmap::of(&[15, 25]);
    /// let bitmap2 = Bitmap::of(&[25, 35]);
    ///
    /// let bitmap3 = Bitmap::fast_xor(&[&bitmap1, &bitmap2]);
    ///
    /// assert_eq!(bitmap3.cardinality(), 2);
    /// assert!(bitmap3.contains(15));
    /// assert!(!bitmap3.contains(25));
    /// assert!(bitmap3.contains(35));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_xor_many")]
    #[must_use]
    pub fn fast_xor(bitmaps: &[&Bitmap]) -> Self {
        let mut bms: Vec<*const ffi::roaring_bitmap_s> = bitmaps
            .iter()
            .map(|item| ptr::addr_of!(item.bitmap))
            .collect();

        unsafe { Self::take_heap(ffi::roaring_bitmap_xor_many(bms.len(), bms.as_mut_ptr())) }
    }

    /// Computes the difference between two bitmaps and returns the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap1 = Bitmap::of(&[15, 25]);
    /// let bitmap2 = Bitmap::of(&[25, 35]);
    ///
    /// let bitmap3 = bitmap1.andnot(&bitmap2);
    ///
    /// assert_eq!(bitmap3.cardinality(), 1);
    /// assert!(bitmap3.contains(15));
    /// assert!(!bitmap3.contains(25));
    /// assert!(!bitmap3.contains(35));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_andnot")]
    #[must_use]
    pub fn andnot(&self, other: &Self) -> Self {
        unsafe { Self::take_heap(ffi::roaring_bitmap_andnot(&self.bitmap, &other.bitmap)) }
    }

    /// Computes the difference between two bitmaps and stores the result
    /// in the current bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::of(&[15, 25]);
    /// let bitmap2 = Bitmap::of(&[25, 35]);
    ///
    /// bitmap1.andnot_inplace(&bitmap2);
    ///
    /// assert_eq!(bitmap1.cardinality(), 1);
    /// assert!(bitmap1.contains(15));
    /// assert!(!bitmap1.contains(25));
    /// assert!(!bitmap1.contains(35));
    ///
    /// let mut bitmap3 = Bitmap::of(&[15]);
    /// let bitmap4 = Bitmap::new();
    /// bitmap3.andnot_inplace(&bitmap4);
    /// assert_eq!(bitmap3.cardinality(), 1);
    /// assert!(bitmap3.contains(15));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_andnot_inplace")]
    pub fn andnot_inplace(&mut self, other: &Self) {
        unsafe { ffi::roaring_bitmap_andnot_inplace(&mut self.bitmap, &other.bitmap) }
    }

    /// Negates the bits in the given range
    /// any integer present in this range and in the bitmap is removed.
    /// Returns result as a new bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap1 = Bitmap::of(&[4]);
    ///
    /// let bitmap2 = bitmap1.flip(1..3);
    ///
    /// assert_eq!(bitmap2.cardinality(), 3);
    /// assert!(bitmap2.contains(1));
    /// assert!(bitmap2.contains(2));
    /// assert!(!bitmap2.contains(3));
    /// assert!(bitmap2.contains(4));
    ///
    /// let bitmap3 = bitmap1.flip(1..=5);
    /// assert_eq!(bitmap3.to_vec(), [1, 2, 3, 5])
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_flip")]
    #[must_use]
    pub fn flip<R: RangeBounds<u32>>(&self, range: R) -> Self {
        let (start, end) = range_to_exclusive(range);
        unsafe { Self::take_heap(ffi::roaring_bitmap_flip(&self.bitmap, start, end)) }
    }

    /// Negates the bits in the given range
    /// any integer present in this range and in the bitmap is removed.
    /// Stores the result in the current bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::of(&[4]);
    /// bitmap1.flip_inplace(1..3);
    ///
    /// assert_eq!(bitmap1.cardinality(), 3);
    /// assert!(bitmap1.contains(1));
    /// assert!(bitmap1.contains(2));
    /// assert!(!bitmap1.contains(3));
    /// assert!(bitmap1.contains(4));
    /// bitmap1.flip_inplace(4..=4);
    /// assert_eq!(bitmap1.to_vec(), [1, 2]);
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_flip_inplace")]
    pub fn flip_inplace<R: RangeBounds<u32>>(&mut self, range: R) {
        let (start, end) = range_to_exclusive(range);
        unsafe { ffi::roaring_bitmap_flip_inplace(&mut self.bitmap, start, end) }
    }

    /// Returns a vector containing all of the integers stored in the Bitmap
    /// in sorted order.
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap = Bitmap::of(&[15, 25]);
    ///
    /// assert_eq!(bitmap.to_vec(), [15, 25]);
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_to_uint32_array")]
    #[must_use]
    pub fn to_vec(&self) -> Vec<u32> {
        let bitmap_size: usize = self.cardinality().try_into().unwrap();

        let mut buffer: Vec<u32> = Vec::with_capacity(bitmap_size);
        unsafe {
            ffi::roaring_bitmap_to_uint32_array(&self.bitmap, buffer.as_mut_ptr());
            buffer.set_len(bitmap_size);
        }
        buffer
    }

    /// Computes the serialized size in bytes of the Bitmap in format `S`.
    #[inline]
    #[must_use]
    pub fn get_serialized_size_in_bytes<S: Serializer>(&self) -> usize {
        S::get_serialized_size_in_bytes(self)
    }

    /// Serializes a bitmap to a slice of bytes in format `S`.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::{Bitmap, Portable};
    ///
    /// let original_bitmap: Bitmap = (1..5).collect();
    ///
    /// let serialized_buffer = original_bitmap.serialize::<Portable>();
    ///
    /// let deserialized_bitmap = Bitmap::deserialize::<Portable>(&serialized_buffer);
    ///
    /// assert_eq!(original_bitmap, deserialized_bitmap);
    /// ```
    #[inline]
    #[must_use]
    pub fn serialize<S: Serializer>(&self) -> Vec<u8> {
        let mut dst = Vec::new();
        self.serialize_into::<S>(&mut dst);
        dst
    }

    /// Serializes a bitmap to a slice of bytes in format `S`, re-using existing capacity
    ///
    /// `dst` is not cleared, data is added after any existing data. Returns the added slice of `dst`.
    /// If `dst` is empty, it is guaranteed to hold only the serialized data after this call
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::{Bitmap, Portable};
    ///
    /// let original_bitmap_1: Bitmap = (1..5).collect();
    /// let original_bitmap_2: Bitmap = (1..10).collect();
    ///
    /// let mut data = Vec::new();
    /// for bitmap in [original_bitmap_1, original_bitmap_2] {
    ///     data.clear();
    ///     bitmap.serialize_into::<Portable>(&mut data);
    ///     // do something with data
    /// }
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_portable_serialize")]
    pub fn serialize_into<'a, S: Serializer>(&self, dst: &'a mut Vec<u8>) -> &'a [u8] {
        S::serialize_into(self, dst)
    }

    /// Given a serialized bitmap as slice of bytes in format `S`, returns a `Bitmap` instance.
    /// See example of [`Self::serialize`] function.
    ///
    /// On invalid input returns None.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::{Bitmap, Portable};
    ///
    /// let original_bitmap: Bitmap = (1..5).collect();
    /// let serialized_buffer = original_bitmap.serialize::<Portable>();
    ///
    /// let deserialized_bitmap = Bitmap::try_deserialize::<Portable>(&serialized_buffer);
    /// assert_eq!(original_bitmap, deserialized_bitmap.unwrap());
    ///
    /// let invalid_buffer: Vec<u8> = vec![3];
    /// let deserialized_bitmap = Bitmap::try_deserialize::<Portable>(&invalid_buffer);
    /// assert!(deserialized_bitmap.is_none());
    /// ```
    #[inline]
    #[must_use]
    pub fn try_deserialize<D: Deserializer>(buffer: &[u8]) -> Option<Self> {
        D::try_deserialize(buffer)
    }

    /// Given a serialized bitmap as slice of bytes in format `S `, returns a bitmap instance.
    /// See example of [`Self::serialize`] function.
    ///
    /// On invalid input returns empty bitmap.
    #[inline]
    pub fn deserialize<D: Deserializer>(buffer: &[u8]) -> Self {
        Self::try_deserialize::<D>(buffer).unwrap_or_else(Bitmap::new)
    }

    /// Creates a new bitmap from a slice of u32 integers
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let elements = vec![1, 2];
    ///
    /// let bitmap = Bitmap::of(&elements);
    ///
    /// let mut bitmap2 = Bitmap::new();
    ///
    /// for element in &elements {
    ///     bitmap2.add(*element);
    /// }
    ///
    /// assert!(bitmap.contains(1));
    /// assert!(bitmap.contains(2));
    /// assert!(!bitmap.contains(3));
    /// assert_eq!(bitmap, bitmap2);
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_of_ptr")]
    #[must_use]
    pub fn of(elements: &[u32]) -> Self {
        // This does the same as `roaring_bitmap_of_ptr`, but that also allocates the bitmap itself
        let mut bitmap = Self::new();
        bitmap.add_many(elements);
        bitmap
    }

    /// Create a new bitmap with all values in `range`
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ops::Bound;
    /// use croaring::Bitmap;
    ///
    /// let bitmap1 = Bitmap::from_range(5..10);
    /// assert_eq!(bitmap1.to_vec(), [5, 6, 7, 8, 9]);
    ///
    /// let bitmap2 = Bitmap::from_range(5..=7);
    /// assert_eq!(bitmap2.to_vec(), [5, 6, 7]);
    ///
    /// let bitmap3 = Bitmap::from_range((Bound::Excluded(2), Bound::Excluded(6)));
    /// assert_eq!(bitmap3.to_vec(), [3, 4, 5]);
    #[inline]
    #[doc(alias = "roaring_bitmap_from_range")]
    pub fn from_range<R: RangeBounds<u32>>(range: R) -> Self {
        let mut result = Self::new();
        result.add_range(range);
        result
    }

    /// Create a new bitmap with all values in `range` which are a multiple of `step` away from the lower bound
    ///
    /// If `step` is zero or there are no values which are a multiple of `step` away from the lower bound
    /// within `range`, an empty bitmap is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ops::Bound;
    /// use croaring::Bitmap;
    ///
    /// let bitmap = Bitmap::from_range_with_step(0..10, 3);
    /// assert_eq!(bitmap.to_vec(), [0, 3, 6, 9]);
    ///
    /// // empty ranges
    /// assert_eq!(Bitmap::from_range_with_step(0..0, 1), Bitmap::new());
    /// assert_eq!(Bitmap::from_range_with_step(100..=0, 1), Bitmap::new());
    ///
    /// // Step of zero
    /// assert_eq!(Bitmap::from_range_with_step(0..100, 0), Bitmap::new());
    ///
    /// // No values of step in range
    /// let bitmap = Bitmap::from_range_with_step((Bound::Excluded(0), Bound::Included(10)), 100);
    /// assert_eq!(bitmap, Bitmap::new());
    /// let bitmap = Bitmap::from_range_with_step((Bound::Excluded(u32::MAX), Bound::Included(u32::MAX)), 1);
    /// assert_eq!(bitmap, Bitmap::new());
    ///
    /// // Exclusive ranges still step from the start, but do not include it
    /// let bitmap = Bitmap::from_range_with_step((Bound::Excluded(10), Bound::Included(30)), 10);
    /// assert_eq!(bitmap.to_vec(), [20, 30]);
    ///
    /// // Ranges including max value
    /// let bitmap = Bitmap::from_range_with_step((u32::MAX - 1)..=u32::MAX, 1);
    /// assert_eq!(bitmap.to_vec(), vec![u32::MAX - 1, u32::MAX]);
    ///
    /// let bitmap = Bitmap::from_range_with_step((u32::MAX - 1)..=u32::MAX, 3);
    /// assert_eq!(bitmap.to_vec(), vec![u32::MAX - 1]);
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_from_range")]
    pub fn from_range_with_step<R: RangeBounds<u32>>(range: R, step: u32) -> Self {
        // This can't use `range_to_exclusive` because when the start is excluded, we want
        // to start at the next step, not one more
        let start = match range.start_bound() {
            Bound::Included(&i) => u64::from(i),
            Bound::Excluded(&i) => u64::from(i) + u64::from(step),
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&i) => u64::from(i) + 1,
            Bound::Excluded(&i) => u64::from(i),
            Bound::Unbounded => u64::MAX,
        };
        unsafe {
            let result = ffi::roaring_bitmap_from_range(start, end, step);
            if result.is_null() {
                Self::new()
            } else {
                Self::take_heap(result)
            }
        }
    }

    /// Shrink the memory allocation of the bitmap if needed
    ///
    /// Returns the number of bytes saved
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::with_container_capacity(10);
    /// let saved_bytes = bitmap.shrink_to_fit();
    /// assert!(saved_bytes > 0);
    /// let more_saved_bytes = bitmap.shrink_to_fit();
    /// assert_eq!(more_saved_bytes, 0);
    #[inline]
    #[doc(alias = "roaring_bitmap_shrink_to_fit")]
    pub fn shrink_to_fit(&mut self) -> usize {
        unsafe { ffi::roaring_bitmap_shrink_to_fit(&mut self.bitmap) }
    }

    /// Compresses of the bitmap. Returns true if the bitmap was modified.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::{Bitmap, Portable};
    ///
    /// let mut bitmap: Bitmap = (100..1000).collect();
    ///
    /// assert_eq!(bitmap.cardinality(), 900);
    /// let old_size = bitmap.get_serialized_size_in_bytes::<Portable>();
    /// assert!(bitmap.run_optimize());
    /// let new_size = bitmap.get_serialized_size_in_bytes::<Portable>();
    /// assert!(new_size < old_size);
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_run_optimize")]
    pub fn run_optimize(&mut self) -> bool {
        unsafe { ffi::roaring_bitmap_run_optimize(&mut self.bitmap) }
    }

    /// Removes run-length encoding even when it is more space efficient. Returns
    /// true if a change was applied.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap: Bitmap = (100..1000).collect();
    ///
    /// assert_eq!(bitmap.cardinality(), 900);
    ///
    /// bitmap.run_optimize();
    ///
    /// assert!(bitmap.remove_run_compression());
    /// assert!(!bitmap.remove_run_compression());
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_remove_run_compression")]
    pub fn remove_run_compression(&mut self) -> bool {
        unsafe { ffi::roaring_bitmap_remove_run_compression(&mut self.bitmap) }
    }

    /// Returns true if the Bitmap is empty.
    /// Faster than doing: bitmap.cardinality() == 0)
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::new();
    ///
    /// assert!(bitmap.is_empty());
    ///
    /// bitmap.add(1);
    ///
    /// assert!(!bitmap.is_empty());
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_is_empty")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        unsafe { ffi::roaring_bitmap_is_empty(&self.bitmap) }
    }

    /// Return true if all the elements of Self are in &other.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap1: Bitmap = (5..10).collect();
    /// let bitmap2: Bitmap = (5..8).collect();
    /// let bitmap3: Bitmap = (5..10).collect();
    /// let bitmap4: Bitmap = (9..11).collect();
    ///
    /// assert!(bitmap2.is_subset(&bitmap1));
    /// assert!(bitmap3.is_subset(&bitmap1));
    /// assert!(!bitmap4.is_subset(&bitmap1));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_is_subset")]
    #[must_use]
    pub fn is_subset(&self, other: &Self) -> bool {
        unsafe { ffi::roaring_bitmap_is_subset(&self.bitmap, &other.bitmap) }
    }

    /// Return true if all the elements of Self are in &other and &other is strictly greater
    /// than Self.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap1: Bitmap = (5..9).collect();
    /// let bitmap2: Bitmap = (5..8).collect();
    /// let bitmap3: Bitmap = (5..10).collect();
    /// let bitmap4: Bitmap = (9..11).collect();
    ///
    /// assert!(bitmap2.is_subset(&bitmap1));
    /// assert!(!bitmap3.is_subset(&bitmap1));
    /// assert!(!bitmap4.is_subset(&bitmap1));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_is_strict_subset")]
    #[must_use]
    pub fn is_strict_subset(&self, other: &Self) -> bool {
        unsafe { ffi::roaring_bitmap_is_strict_subset(&self.bitmap, &other.bitmap) }
    }

    /// Return true if Self and &other intersect
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap1: Bitmap = (1..5).collect();
    /// let bitmap2: Bitmap = (5..9).collect();
    /// let bitmap3: Bitmap = (3..7).collect();
    ///
    /// assert_eq!(bitmap1.intersect(&bitmap2), false);
    /// assert_eq!(bitmap1.intersect(&bitmap3), true);
    /// assert_eq!(bitmap2.intersect(&bitmap3), true);
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_intersect")]
    #[must_use]
    pub fn intersect(&self, other: &Self) -> bool {
        unsafe { ffi::roaring_bitmap_intersect(&self.bitmap, &other.bitmap) }
    }

    /// Check if a bitmap has any values set in `range`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap = Bitmap::of(&[1, 100, 101, u32::MAX]);
    ///
    /// assert!(bitmap.intersect_with_range(0..10));
    /// assert!(!bitmap.intersect_with_range(2..100));
    /// assert!(bitmap.intersect_with_range(999..=u32::MAX));
    ///
    /// // Empty ranges
    /// assert!(!bitmap.intersect_with_range(100..100));
    /// assert!(!bitmap.intersect_with_range(100..0));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_intersect_with_range")]
    pub fn intersect_with_range<R: RangeBounds<u32>>(&self, range: R) -> bool {
        let (start, end) = range_to_exclusive(range);
        unsafe { ffi::roaring_bitmap_intersect_with_range(&self.bitmap, start, end) }
    }

    /// Return the Jaccard index between Self and &other
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap1: Bitmap = (1..5).collect();
    /// let bitmap2: Bitmap = (5..9).collect();
    /// let bitmap3: Bitmap = (3..9).collect();
    ///
    /// assert_eq!(bitmap1.jaccard_index(&bitmap2), 0.0);
    /// assert_eq!(bitmap1.jaccard_index(&bitmap3), 0.25);
    /// assert_eq!(bitmap2.jaccard_index(&bitmap3), 0.6666666666666666);
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_jaccard_index")]
    #[must_use]
    pub fn jaccard_index(&self, other: &Self) -> f64 {
        unsafe { ffi::roaring_bitmap_jaccard_index(&self.bitmap, &other.bitmap) }
    }

    /// Return the size of the intersection between Self and &other
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap1 = Bitmap::of(&[1]);
    /// let bitmap2 = Bitmap::of(&[1, 2]);
    ///
    /// assert_eq!(bitmap1.and_cardinality(&bitmap2), 1);
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_and_cardinality")]
    #[must_use]
    pub fn and_cardinality(&self, other: &Self) -> u64 {
        unsafe { ffi::roaring_bitmap_and_cardinality(&self.bitmap, &other.bitmap) }
    }

    /// Return the size of the union between Self and &other
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap1 = Bitmap::of(&[15]);
    /// let bitmap2 = Bitmap::of(&[25]);
    ///
    /// assert_eq!(bitmap1.or_cardinality(&bitmap2), 2);
    #[inline]
    #[doc(alias = "roaring_bitmap_or_cardinality")]
    #[must_use]
    pub fn or_cardinality(&self, other: &Self) -> u64 {
        unsafe { ffi::roaring_bitmap_or_cardinality(&self.bitmap, &other.bitmap) }
    }

    /// Return the size of the difference between Self and &other
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap1 = Bitmap::of(&[15, 25]);
    /// let bitmap2 = Bitmap::of(&[25, 35]);
    ///
    /// assert_eq!(bitmap1.andnot_cardinality(&bitmap2), 1);
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_andnot_cardinality")]
    #[must_use]
    pub fn andnot_cardinality(&self, other: &Self) -> u64 {
        unsafe { ffi::roaring_bitmap_andnot_cardinality(&self.bitmap, &other.bitmap) }
    }

    /// Return the size of the symmetric difference between Self and &other
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap1 = Bitmap::of(&[15, 25]);
    /// let bitmap2 = Bitmap::of(&[25, 35]);
    ///
    /// assert_eq!(bitmap1.xor_cardinality(&bitmap2), 2);
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_xor_cardinality")]
    #[must_use]
    pub fn xor_cardinality(&self, other: &Self) -> u64 {
        unsafe { ffi::roaring_bitmap_xor_cardinality(&self.bitmap, &other.bitmap) }
    }

    /// Returns the smallest value in the set.
    ///
    /// Returns `None` if the set is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap: Bitmap = (5..10).collect();
    /// let empty_bitmap: Bitmap = Bitmap::new();
    ///
    /// assert_eq!(bitmap.minimum(), Some(5));
    /// assert_eq!(empty_bitmap.minimum(), None);
    ///
    /// bitmap.add(3);
    ///
    /// assert_eq!(bitmap.minimum(), Some(3));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_minimum")]
    #[must_use]
    pub fn minimum(&self) -> Option<u32> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { ffi::roaring_bitmap_minimum(&self.bitmap) })
        }
    }

    /// Returns the greatest value in the set.
    ///
    /// Returns `None` if the set is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap: Bitmap = (5..10).collect();
    /// let empty_bitmap: Bitmap = Bitmap::new();
    ///
    /// assert_eq!(bitmap.maximum(), Some(9));
    /// assert_eq!(empty_bitmap.maximum(), None);
    ///
    /// bitmap.add(15);
    ///
    /// assert_eq!(bitmap.maximum(), Some(15));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_maximum")]
    #[must_use]
    pub fn maximum(&self) -> Option<u32> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { ffi::roaring_bitmap_maximum(&self.bitmap) })
        }
    }

    /// Rank returns the number of values smaller or equal to x.
    ///
    /// For a similar function which also checks if x is in the set, see [position][Self::position].
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap: Bitmap = (5..10).collect();
    ///
    /// assert_eq!(bitmap.rank(8), 4);
    ///
    /// bitmap.add(15);
    ///
    /// assert_eq!(bitmap.rank(11), 5);
    /// assert_eq!(bitmap.rank(15), 6);
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_rank")]
    #[must_use]
    pub fn rank(&self, x: u32) -> u64 {
        unsafe { ffi::roaring_bitmap_rank(&self.bitmap, x) }
    }

    /// Returns the index of x in the given roaring bitmap.
    ///
    /// If the roaring bitmap doesn't contain x, this function will return None.
    /// The difference with the [rank][Self::rank] function is that this function
    /// will return None when x is not the element of roaring bitmap, but the rank
    /// function will return the the number of items less than x, and would require
    /// a call to [contains][Self::contains] to check if x is in the roaring bitmap.
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap: Bitmap = Bitmap::from_range(5..10);
    /// assert_eq!(bitmap.position(4), None);
    /// assert_eq!(bitmap.position(5), Some(0));
    /// assert_eq!(bitmap.position(9), Some(4));
    /// assert_eq!(bitmap.position(10), None);
    /// assert_eq!(bitmap.position(9999), None);
    ///
    /// // rank returns the number of values smaller or equal to x, so it always returns a value, and
    /// // returns `position + 1` when x is contained in the bitmap.
    /// assert_eq!(bitmap.rank(4), 0);
    /// assert_eq!(bitmap.rank(5), 1);
    /// assert_eq!(bitmap.rank(9), 5);
    /// assert_eq!(bitmap.rank(10), 5);
    /// assert_eq!(bitmap.rank(9999), 5);
    ///
    /// let pos = bitmap.position(7).unwrap();
    /// assert_eq!(bitmap.select(pos), Some(7));
    /// ```
    #[inline]
    #[doc(alias = "index")]
    #[doc(alias = "roaring_bitmap_get_index")]
    #[must_use]
    pub fn position(&self, x: u32) -> Option<u32> {
        let index = unsafe { ffi::roaring_bitmap_get_index(&self.bitmap, x) };
        if index == -1 {
            None
        } else {
            Some(u32::try_from(index).unwrap())
        }
    }

    /// Select returns the element having the designated position, if it exists
    ///
    /// If the size of the roaring bitmap is strictly greater than pos,
    /// then this function returns element of given rank wrapped in Some.
    /// Otherwise, it returns None.
    ///
    /// To do the inverse operation (given an element, find its position), use the
    /// [position][Self::position] function, or the [rank][Self::rank] function.
    ///
    /// Note that the [rank][Self::rank] function is inclusive: it returns the number of values
    /// smaller or equal to x, when `x` is contained in the bitmap, it returns
    /// `position + 1`.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap: Bitmap = (5..10).collect();
    ///
    /// assert_eq!(bitmap.select(0), Some(5));
    /// assert_eq!(bitmap.select(1), Some(6));
    /// assert_eq!(bitmap.select(2), Some(7));
    /// assert_eq!(bitmap.select(3), Some(8));
    /// assert_eq!(bitmap.select(4), Some(9));
    /// assert_eq!(bitmap.select(5), None);
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_select")]
    #[must_use]
    pub fn select(&self, position: u32) -> Option<u32> {
        let mut element: u32 = 0;
        let result = unsafe { ffi::roaring_bitmap_select(&self.bitmap, position, &mut element) };

        if result {
            Some(element)
        } else {
            None
        }
    }

    /// Returns statistics about the composition of a roaring bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap: Bitmap = (1..100).collect();
    /// let statistics = bitmap.statistics();
    ///
    /// assert_eq!(statistics.n_containers, 1);
    /// assert_eq!(statistics.n_array_containers, 1);
    /// assert_eq!(statistics.n_run_containers, 0);
    /// assert_eq!(statistics.n_bitset_containers, 0);
    /// assert_eq!(statistics.n_values_array_containers, 99);
    /// assert_eq!(statistics.n_values_run_containers, 0);
    /// assert_eq!(statistics.n_values_bitset_containers, 0);
    /// assert_eq!(statistics.n_bytes_array_containers, 198);
    /// assert_eq!(statistics.n_bytes_run_containers, 0);
    /// assert_eq!(statistics.n_bytes_bitset_containers, 0);
    /// assert_eq!(statistics.max_value, 99);
    /// assert_eq!(statistics.min_value, 1);
    /// assert_eq!(statistics.sum_value, 4950);
    /// assert_eq!(statistics.cardinality, 99);
    ///
    /// bitmap.run_optimize();
    /// let statistics = bitmap.statistics();
    ///
    /// assert_eq!(statistics.n_containers, 1);
    /// assert_eq!(statistics.n_array_containers, 0);
    /// assert_eq!(statistics.n_run_containers, 1);
    /// assert_eq!(statistics.n_bitset_containers, 0);
    /// assert_eq!(statistics.n_values_array_containers, 0);
    /// assert_eq!(statistics.n_values_run_containers, 99);
    /// assert_eq!(statistics.n_values_bitset_containers, 0);
    /// assert_eq!(statistics.n_bytes_array_containers, 0);
    /// assert_eq!(statistics.n_bytes_run_containers, 6);
    /// assert_eq!(statistics.n_bytes_bitset_containers, 0);
    /// assert_eq!(statistics.max_value, 99);
    /// assert_eq!(statistics.min_value, 1);
    /// assert_eq!(statistics.sum_value, 4950);
    /// assert_eq!(statistics.cardinality, 99);
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_statistics")]
    #[must_use]
    pub fn statistics(&self) -> Statistics {
        let mut statistics: ffi::roaring_statistics_s = unsafe { ::std::mem::zeroed() };

        unsafe { ffi::roaring_bitmap_statistics(&self.bitmap, &mut statistics) };

        statistics
    }

    /// Store the bitmap to a bitset
    ///
    /// This can be useful for those who need the performance and simplicity of a standard bitset.
    ///
    /// # Errors
    ///
    /// This function will return None on allocation failure
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitmap;
    /// let bitmap = Bitmap::from_range(0..100);
    /// let bitset = bitmap.to_bitset().unwrap();
    /// assert_eq!(bitset.count(), 100);
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_to_bitset")]
    #[must_use]
    pub fn to_bitset(&self) -> Option<Bitset> {
        let mut bitset = Bitset::new();
        let success = unsafe { ffi::roaring_bitmap_to_bitset(&self.bitmap, bitset.as_raw_mut()) };
        success.then_some(bitset)
    }

    /// Ensure the bitmap is internally valid
    ///
    /// This is useful for development, but is not needed for normal use:
    /// bitmaps should _always_ be internally valid.
    ///
    /// # Errors
    ///
    /// Returns an error if the bitmap is not valid, with a description of the problem.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap = Bitmap::from_range(0..100);
    /// bitmap.internal_validate().unwrap();
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_internal_validate")]
    #[doc(hidden)]
    pub fn internal_validate(&self) -> Result<(), String> {
        let mut error_str = ptr::null();
        let valid = unsafe { ffi::roaring_bitmap_internal_validate(&self.bitmap, &mut error_str) };
        if valid {
            Ok(())
        } else {
            if error_str.is_null() {
                return Err(String::from("Unknown error"));
            }
            let reason = unsafe { CStr::from_ptr(error_str) };
            Err(reason.to_string_lossy().into_owned())
        }
    }
}

fn range_to_inclusive<R: RangeBounds<u32>>(range: R) -> (u32, u32) {
    let start = match range.start_bound() {
        Bound::Included(&i) => i,
        Bound::Excluded(&i) => match i.checked_add(1) {
            Some(i) => i,
            None => return (1, 0),
        },
        Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        Bound::Included(&i) => i,
        Bound::Excluded(&i) => match i.checked_sub(1) {
            Some(i) => i,
            None => return (1, 0),
        },
        Bound::Unbounded => u32::MAX,
    };
    (start, end)
}

fn range_to_exclusive<R: RangeBounds<u32>>(range: R) -> (u64, u64) {
    let start = match range.start_bound() {
        Bound::Included(&i) => u64::from(i),
        Bound::Excluded(&i) => u64::from(i) + 1,
        Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        Bound::Included(&i) => u64::from(i) + 1,
        Bound::Excluded(&i) => u64::from(i),
        Bound::Unbounded => u64::MAX,
    };
    (start, end)
}
