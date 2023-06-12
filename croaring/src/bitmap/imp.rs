use ffi::roaring_bitmap_t;
use std::convert::TryInto;
use std::ffi::c_char;
use std::mem;
use std::ops::{Bound, RangeBounds};

use super::{Bitmap, Statistics};

impl Bitmap {
    #[inline]
    #[allow(clippy::assertions_on_constants)]
    unsafe fn take_heap(p: *mut roaring_bitmap_t) -> Self {
        // Based heavily on the `roaring.hh` cpp header from croaring

        assert!(!p.is_null());
        let result = Self { bitmap: *p };
        // This depends somewhat heavily on the implementation of croaring,
        // In particular, that `roaring_bitmap_t` doesn't store any pointers into itself
        // (it can be moved safely), and can be freed with `free`, without freeing the underlying
        // containers and auxiliary data. Ensure this is still valid every time we update
        // the version of croaring.
        const _: () = assert!(
            ffi::ROARING_VERSION_MAJOR == 1
                && ffi::ROARING_VERSION_MINOR == 3
                && ffi::ROARING_VERSION_REVISION == 0
        );
        ffi::roaring_free(p as *mut _);
        result
    }

    /// Creates a new bitmap (initially empty)
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap = Bitmap::create();
    ///
    /// assert!(bitmap.is_empty());
    /// ```
    #[inline]
    pub fn create() -> Self {
        Self::create_with_capacity(0)
    }

    /// Creates a new bitmap (initially empty) with a provided
    /// container-storage capacity (it is a performance hint).
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap = Bitmap::create_with_capacity(100_000);
    ///
    /// assert!(bitmap.is_empty());
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_init_with_capacity")]
    pub fn create_with_capacity(capacity: u32) -> Self {
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
    /// let mut bitmap = Bitmap::create();
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
    /// let mut bitmap = Bitmap::create();
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
    /// let mut bitmap = Bitmap::create();
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
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add_range((1..3));
    ///
    /// assert!(!bitmap1.is_empty());
    /// assert!(bitmap1.contains(1));
    /// assert!(bitmap1.contains(2));
    /// assert!(!bitmap1.contains(3));
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add_range((3..1));
    /// assert!(bitmap2.is_empty());
    ///
    /// let mut bitmap3 = Bitmap::create();
    /// bitmap3.add_range((3..3));
    /// assert!(bitmap3.is_empty());
    ///
    /// let mut bitmap4 = Bitmap::create();
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
    /// let mut bitmap = Bitmap::create();
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
    /// let mut bitmap = Bitmap::create();
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
    /// let mut bitmap = Bitmap::create();
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
    /// let mut bitmap = Bitmap::create();
    /// bitmap.add(1);
    /// assert!(bitmap.remove_checked(1));
    /// assert!(!bitmap.remove_checked(1));
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_remove_checked")]
    pub fn remove_checked(&mut self, element: u32) -> bool {
        unsafe { ffi::roaring_bitmap_remove_checked(&mut self.bitmap, element) }
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
    pub fn fast_or(bitmaps: &[&Bitmap]) -> Self {
        let mut bms: Vec<*const ffi::roaring_bitmap_s> = bitmaps
            .iter()
            .map(|item| &item.bitmap as *const _)
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
    pub fn fast_or_heap(bitmaps: &[&Bitmap]) -> Self {
        let mut bms: Vec<*const ffi::roaring_bitmap_s> = bitmaps
            .iter()
            .map(|item| &item.bitmap as *const _)
            .collect();

        unsafe {
            Self::take_heap(ffi::roaring_bitmap_or_many_heap(
                bms.len() as u32,
                bms.as_mut_ptr(),
            ))
        }
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
    pub fn fast_xor(bitmaps: &[&Bitmap]) -> Self {
        let mut bms: Vec<*const ffi::roaring_bitmap_s> = bitmaps
            .iter()
            .map(|item| &item.bitmap as *const _)
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
    /// let bitmap4 = Bitmap::create();
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
    pub fn to_vec(&self) -> Vec<u32> {
        let bitmap_size: usize = self.cardinality().try_into().unwrap();

        let mut buffer: Vec<u32> = Vec::with_capacity(bitmap_size);
        unsafe {
            ffi::roaring_bitmap_to_uint32_array(&self.bitmap, buffer.as_mut_ptr());
            buffer.set_len(bitmap_size);
        }
        buffer
    }

    /// Computes the serialized size in bytes of the Bitmap.
    #[inline]
    #[doc(alias = "roaring_bitmap_portable_size_in_bytes")]
    pub fn get_serialized_size_in_bytes(&self) -> usize {
        unsafe { ffi::roaring_bitmap_portable_size_in_bytes(&self.bitmap) }
    }

    /// Computes the serialized size in bytes of the Bitmap for the frozen format.
    #[inline]
    #[doc(alias = "roaring_bitmap_frozen_size_in_bytes")]
    pub fn get_frozen_serialized_size_in_bytes(&self) -> usize {
        unsafe { ffi::roaring_bitmap_frozen_size_in_bytes(&self.bitmap) }
    }

    /// Serializes a bitmap to a slice of bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let original_bitmap: Bitmap = (1..5).collect();
    ///
    /// let serialized_buffer = original_bitmap.serialize();
    ///
    /// let deserialized_bitmap = Bitmap::deserialize(&serialized_buffer);
    ///
    /// assert_eq!(original_bitmap, deserialized_bitmap);
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_portable_serialize")]
    pub fn serialize(&self) -> Vec<u8> {
        let mut dst = Vec::new();
        self.serialize_into(&mut dst);
        dst
    }

    /// Serializes a bitmap to a slice of bytes, re-using existing capacity
    ///
    /// `dst` is not cleared, data is added after any existing data. Returns the added slice of `dst`.
    /// If `dst` is empty, it is guaranteed to hold only the serialized data after this call
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let original_bitmap_1: Bitmap = (1..5).collect();
    /// let original_bitmap_2: Bitmap = (1..10).collect();
    ///
    /// let mut data = Vec::new();
    /// for bitmap in [original_bitmap_1, original_bitmap_2] {
    ///     data.clear();
    ///     bitmap.serialize_into(&mut data);
    ///     // do something with data
    /// }
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_portable_serialize")]
    pub fn serialize_into<'a>(&self, dst: &'a mut Vec<u8>) -> &'a [u8] {
        let len = self.get_serialized_size_in_bytes();

        dst.reserve(len);
        let total_len = dst.len().checked_add(len).unwrap();

        unsafe {
            ffi::roaring_bitmap_portable_serialize(
                &self.bitmap,
                dst.spare_capacity_mut().as_mut_ptr().cast::<c_char>(),
            );
            dst.set_len(total_len);
        }

        dst
    }

    /// Serialize into the "frozen" format
    ///
    /// This has an odd API because it always returns a slice which is aligned to 32 bytes:
    /// This means the returned slice may not start exactly at the beginning of the passed Vec
    #[doc(alias = "roaring_bitmap_frozen_serialize")]
    pub fn serialize_frozen_into<'a>(&self, dst: &'a mut Vec<u8>) -> &'a [u8] {
        const REQUIRED_ALIGNMENT: usize = 32;
        let len = self.get_frozen_serialized_size_in_bytes();

        let offset = dst.len();
        // Need to be able to add up to 31 extra bytes to align to 32 bytes
        dst.reserve(len.checked_add(REQUIRED_ALIGNMENT - 1).unwrap());

        let extra_offset = match (dst.as_ptr() as usize) % REQUIRED_ALIGNMENT {
            0 => 0,
            r => REQUIRED_ALIGNMENT - r,
        };
        let offset = offset.checked_add(extra_offset).unwrap();
        let total_len = offset.checked_add(len).unwrap();
        debug_assert!(dst.capacity() >= total_len);

        // we must initialize up to offset
        dst.resize(offset, 0);

        unsafe {
            ffi::roaring_bitmap_frozen_serialize(
                &self.bitmap,
                dst.as_mut_ptr().add(offset).cast::<c_char>(),
            );
            dst.set_len(total_len);
        }

        &dst[offset..total_len]
    }

    /// Given a serialized bitmap as slice of bytes returns a bitmap instance.
    /// See example of [`Self::serialize`] function.
    ///
    /// On invalid input returns None.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let original_bitmap: Bitmap = (1..5).collect();
    /// let serialized_buffer = original_bitmap.serialize();
    ///
    /// let deserialized_bitmap = Bitmap::try_deserialize(&serialized_buffer);
    /// assert_eq!(original_bitmap, deserialized_bitmap.unwrap());
    ///
    /// let invalid_buffer: Vec<u8> = vec![3];
    /// let deserialized_bitmap = Bitmap::try_deserialize(&invalid_buffer);
    /// assert!(deserialized_bitmap.is_none());
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_portable_deserialize_safe")]
    pub fn try_deserialize(buffer: &[u8]) -> Option<Self> {
        unsafe {
            let bitmap = ffi::roaring_bitmap_portable_deserialize_safe(
                buffer.as_ptr() as *const c_char,
                buffer.len(),
            );

            if !bitmap.is_null() {
                Some(Self::take_heap(bitmap))
            } else {
                None
            }
        }
    }

    /// Given a serialized bitmap as slice of bytes returns a bitmap instance.
    /// See example of [`Self::serialize`] function.
    ///
    /// On invalid input returns empty bitmap.
    #[inline]
    pub fn deserialize(buffer: &[u8]) -> Self {
        Self::try_deserialize(buffer).unwrap_or_else(Bitmap::create)
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
    /// let mut bitmap2 = Bitmap::create();
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
    pub fn of(elements: &[u32]) -> Self {
        unsafe {
            Self::take_heap(ffi::roaring_bitmap_of_ptr(
                elements.len(),
                elements.as_ptr(),
            ))
        }
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
        let mut result = Self::create();
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
    /// assert_eq!(Bitmap::from_range_with_step(0..0, 1), Bitmap::create());
    /// assert_eq!(Bitmap::from_range_with_step(100..=0, 1), Bitmap::create());
    ///
    /// // Step of zero
    /// assert_eq!(Bitmap::from_range_with_step(0..100, 0), Bitmap::create());
    ///
    /// // No values of step in range
    /// let bitmap = Bitmap::from_range_with_step((Bound::Excluded(0), Bound::Included(10)), 100);
    /// assert_eq!(bitmap, Bitmap::create());
    /// let bitmap = Bitmap::from_range_with_step((Bound::Excluded(u32::MAX), Bound::Included(u32::MAX)), 1);
    /// assert_eq!(bitmap, Bitmap::create());
    ///
    /// // Exclusive ranges still step from the start, but do not include it
    /// let bitmap = Bitmap::from_range_with_step((Bound::Excluded(10), Bound::Included(30)), 10);
    /// assert_eq!(bitmap.to_vec(), [20, 30]);
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
                Self::create()
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
    /// let mut bitmap = Bitmap::create_with_capacity(10);
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
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap: Bitmap = (100..1000).collect();
    ///
    /// assert_eq!(bitmap.cardinality(), 900);
    /// let old_size = bitmap.get_serialized_size_in_bytes();
    /// assert!(bitmap.run_optimize());
    /// let new_size = bitmap.get_serialized_size_in_bytes();
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
    /// let mut bitmap = Bitmap::create();
    ///
    /// assert!(bitmap.is_empty());
    ///
    /// bitmap.add(1);
    ///
    /// assert!(!bitmap.is_empty());
    /// ```
    #[inline]
    #[doc(alias = "roaring_bitmap_is_empty")]
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
    /// let empty_bitmap: Bitmap = Bitmap::create();
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
    /// let empty_bitmap: Bitmap = Bitmap::create();
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
    pub fn maximum(&self) -> Option<u32> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { ffi::roaring_bitmap_maximum(&self.bitmap) })
        }
    }

    /// Rank returns the number of values smaller or equal to x.
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
    pub fn rank(&self, x: u32) -> u64 {
        unsafe { ffi::roaring_bitmap_rank(&self.bitmap, x) }
    }

    /// Select returns the element having the designated rank, if it exists
    /// If the size of the roaring bitmap is strictly greater than rank,
    /// then this function returns element of given rank wrapped in Some.
    /// Otherwise, it returns None.
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
    pub fn select(&self, rank: u32) -> Option<u32> {
        let mut element: u32 = 0;
        let result = unsafe { ffi::roaring_bitmap_select(&self.bitmap, rank, &mut element) };

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
    pub fn statistics(&self) -> Statistics {
        let mut statistics: ffi::roaring_statistics_s = unsafe { ::std::mem::zeroed() };

        unsafe { ffi::roaring_bitmap_statistics(&self.bitmap, &mut statistics) };

        statistics
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
