use std::ops::Range;

use super::{ffi, Bitmap, Statistics};

impl Bitmap {
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
        let bitmap = unsafe { ffi::roaring_bitmap_create() };

        Bitmap { bitmap }
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
    pub fn create_with_capacity(capacity: u32) -> Self {
        let bitmap = unsafe { ffi::roaring_bitmap_create_with_capacity(capacity) };

        Bitmap { bitmap }
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
    pub fn add_many(&mut self, elements: &[u32]) {
        unsafe { ffi::roaring_bitmap_add_many(self.bitmap, elements.len(), elements.as_ptr()) }
    }

    /// Add the integer element to the bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::create();
    /// bitmap.add(1);
    ///
    /// assert!(!bitmap.is_empty());
    /// ```
    #[inline]
    pub fn add(&mut self, element: u32) {
        unsafe { ffi::roaring_bitmap_add(self.bitmap, element) }
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
    pub fn add_checked(&mut self, element: u32) -> bool {
        unsafe { ffi::roaring_bitmap_add_checked(self.bitmap, element) }
    }

    /// Add all values in range [range_min, range_max)
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
    /// ```
    #[inline]
    pub fn add_range(&mut self, range: Range<u64>) {
        unsafe { ffi::roaring_bitmap_add_range(self.bitmap, range.start, range.end) }
    }

    /// Remove all values in range [range_min, range_max)
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
    /// ```
    #[inline]
    pub fn remove_range(&mut self, range: Range<u64>) {
        unsafe { ffi::roaring_bitmap_remove_range(self.bitmap, range.start, range.end) }
    }

    /// Add all values in range [range_min, range_max]
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add_range_closed((1..3));
    ///
    /// assert!(!bitmap1.is_empty());
    /// assert!(bitmap1.contains(1));
    /// assert!(bitmap1.contains(2));
    /// assert!(bitmap1.contains(3));
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add_range_closed((3..1));
    /// assert!(bitmap2.is_empty());
    ///
    /// let mut bitmap3 = Bitmap::create();
    /// bitmap3.add_range_closed((3..3));
    /// assert!(!bitmap3.is_empty());
    /// assert!(bitmap3.contains(3));
    /// ```
    #[inline]
    pub fn add_range_closed(&mut self, range: Range<u32>) {
        unsafe { ffi::roaring_bitmap_add_range_closed(self.bitmap, range.start, range.end + 1) }
    }

    /// Remove all values in range [range_min, range_max]
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
    /// bitmap.remove_range_closed((1..3));
    ///
    /// assert!(!bitmap.contains(1));
    /// assert!(!bitmap.contains(2));
    /// assert!(!bitmap.contains(3));
    /// ```
    #[inline]
    pub fn remove_range_closed(&mut self, range: Range<u32>) {
        unsafe { ffi::roaring_bitmap_remove_range_closed(self.bitmap, range.start, range.end) }
    }

    /// Check whether a range of values of range are present
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::create();
    /// bitmap.add_range((1..3));
    /// assert!(bitmap.contains_range((1..3)));
    /// ```
    #[inline]
    pub fn contains_range(&mut self, range: Range<u64>) -> bool {
        unsafe { ffi::roaring_bitmap_contains_range(self.bitmap, range.start, range.end) }
    }

    /// Remove the integer element from the bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::create();
    /// bitmap.add(1);
    /// bitmap.clear();
    ///
    /// assert!(bitmap.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        unsafe { ffi::roaring_bitmap_clear(self.bitmap) }
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
    pub fn remove(&mut self, element: u32) {
        unsafe { ffi::roaring_bitmap_remove(self.bitmap, element) }
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
    pub fn remove_checked(&mut self, element: u32) -> bool {
        unsafe { ffi::roaring_bitmap_remove_checked(self.bitmap, element) }
    }

    /// Contains returns true if the integer element is contained in the bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::create();
    /// bitmap.add(1);
    ///
    /// assert!(bitmap.contains(1));
    /// assert!(!bitmap.contains(2));
    /// ```
    #[inline]
    pub fn contains(&self, element: u32) -> bool {
        unsafe { ffi::roaring_bitmap_contains(self.bitmap, element) }
    }

    /// Returns number of elements in range [range_start, range_end).
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::create();
    /// bitmap.add(1);
    /// bitmap.add(3);
    /// bitmap.add(4);
    ///
    /// assert_eq!(bitmap.range_cardinality((0..1)), 0);
    /// assert_eq!(bitmap.range_cardinality((0..2)), 1);
    /// assert_eq!(bitmap.range_cardinality((2..5)), 2);
    /// assert_eq!(bitmap.range_cardinality((0..5)), 3);
    /// ```
    #[inline]
    pub fn range_cardinality(&self, range: Range<u64>) -> u64 {
        unsafe { ffi::roaring_bitmap_range_cardinality(self.bitmap, range.start, range.end) }
    }

    /// Returns the number of integers contained in the bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::create();
    /// bitmap.add(1);
    ///
    /// assert_eq!(bitmap.cardinality(), 1);
    ///
    /// bitmap.add(2);
    ///
    /// assert_eq!(bitmap.cardinality(), 2);
    /// ```
    #[inline]
    pub fn cardinality(&self) -> u64 {
        unsafe { ffi::roaring_bitmap_get_cardinality(self.bitmap) }
    }

    /// And computes the intersection between two bitmaps and returns the result
    /// as a new bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(1);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(1);
    /// bitmap2.add(2);
    ///
    /// let bitmap3 = bitmap1.and(&bitmap2);
    ///
    /// assert!(bitmap3.contains(1));
    /// assert!(!bitmap3.contains(2));
    /// ```
    #[inline]
    pub fn and(&self, other: &Self) -> Self {
        Bitmap {
            bitmap: unsafe { ffi::roaring_bitmap_and(self.bitmap, other.bitmap) },
        }
    }

    /// Computes the intersection between two bitmaps and stores the result
    /// in the current bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(15);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(25);
    ///
    /// let mut bitmap3 = Bitmap::create();
    /// bitmap3.add(15);
    ///
    /// let mut bitmap4 = Bitmap::create();
    /// bitmap4.add(15);
    /// bitmap4.add(25);
    ///
    /// bitmap1.and_inplace(&bitmap2);
    ///
    /// assert!(bitmap1.cardinality() == 0);
    /// assert!(!bitmap1.contains(15));
    /// assert!(!bitmap1.contains(25));
    ///
    /// bitmap3.and_inplace(&bitmap4);
    ///
    /// assert!(bitmap3.cardinality() == 1);
    /// assert!(bitmap3.contains(15));
    /// assert!(!bitmap3.contains(25));
    /// ```
    #[inline]
    pub fn and_inplace(&mut self, other: &Self) {
        unsafe { ffi::roaring_bitmap_and_inplace(self.bitmap, other.bitmap) }
    }

    /// Or computes the union between two bitmaps and returns the result
    /// as a new bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(15);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(25);
    ///
    /// let bitmap3 = bitmap1.or(&bitmap2);
    ///
    /// assert!(bitmap3.cardinality() == 2);
    /// assert!(bitmap3.contains(15));
    /// assert!(bitmap3.contains(25));
    /// ```
    #[inline]
    pub fn or(&self, other: &Self) -> Self {
        Bitmap {
            bitmap: unsafe { ffi::roaring_bitmap_or(self.bitmap, other.bitmap) },
        }
    }

    /// Computes the union between two bitmaps and stores the result in
    /// the current bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(15);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(25);
    ///
    /// bitmap1.or_inplace(&bitmap2);
    ///
    /// assert!(bitmap1.cardinality() == 2);
    /// assert!(bitmap1.contains(15));
    /// assert!(bitmap1.contains(25));
    /// ```
    #[inline]
    pub fn or_inplace(&mut self, other: &Self) {
        unsafe { ffi::roaring_bitmap_or_inplace(self.bitmap, other.bitmap) }
    }

    /// Computes the union between many bitmaps quickly, as opposed to having
    /// to call or() repeatedly. Returns the result as a new bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(15);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(25);
    ///
    /// let mut bitmap3 = Bitmap::create();
    /// bitmap3.add(35);
    ///
    /// let bitmap4 = Bitmap::fast_or(&[&bitmap1, &bitmap2, &bitmap3]);
    ///
    /// assert_eq!(bitmap4.cardinality(), 3);
    /// assert!(bitmap4.contains(15));
    /// assert!(bitmap4.contains(25));
    /// assert!(bitmap4.contains(25));
    /// ```
    #[inline]
    pub fn fast_or(bitmaps: &[&Bitmap]) -> Self {
        let mut bms: Vec<*const ffi::roaring_bitmap_s> = Vec::with_capacity(bitmaps.len());

        for (i, item) in bitmaps.iter().enumerate() {
            bms.insert(i, item.bitmap);
        }

        Bitmap {
            bitmap: unsafe { ffi::roaring_bitmap_or_many(bms.len(), bms.as_mut_ptr()) },
        }
    }

    /// Compute the union of 'number' bitmaps using a heap. This can
    /// sometimes be faster than Bitmap::fast_or.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(15);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(25);
    ///
    /// let mut bitmap3 = Bitmap::create();
    /// bitmap3.add(35);
    ///
    /// let bitmap4 = Bitmap::fast_or_heap(&[&bitmap1, &bitmap2, &bitmap3]);
    ///
    /// assert_eq!(bitmap4.cardinality(), 3);
    /// assert!(bitmap4.contains(15));
    /// assert!(bitmap4.contains(25));
    /// assert!(bitmap4.contains(25));
    /// ```
    #[inline]
    pub fn fast_or_heap(bitmaps: &[&Bitmap]) -> Self {
        let mut bms: Vec<*const ffi::roaring_bitmap_s> = Vec::with_capacity(bitmaps.len());

        for (i, item) in bitmaps.iter().enumerate() {
            bms.insert(i, item.bitmap);
        }

        Bitmap {
            bitmap: unsafe { ffi::roaring_bitmap_or_many_heap(bms.len() as u32, bms.as_mut_ptr()) },
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
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(15);
    /// bitmap1.add(25);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(25);
    /// bitmap2.add(35);
    ///
    /// let bitmap3 = bitmap1.xor(&bitmap2);
    ///
    /// assert!(bitmap3.cardinality() == 2);
    /// assert!(bitmap3.contains(15));
    /// assert!(!bitmap3.contains(25));
    /// assert!(bitmap3.contains(35));
    /// ```
    #[inline]
    pub fn xor(&self, other: &Self) -> Self {
        Bitmap {
            bitmap: unsafe { ffi::roaring_bitmap_xor(self.bitmap, other.bitmap) },
        }
    }

    /// Inplace version of roaring_bitmap_xor, stores result in current bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(15);
    /// bitmap1.add(25);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(25);
    /// bitmap2.add(35);
    ///
    /// bitmap1.xor_inplace(&bitmap2);
    ///
    /// assert!(bitmap1.cardinality() == 2);
    /// assert!(bitmap1.contains(15));
    /// assert!(!bitmap1.contains(25));
    /// assert!(bitmap1.contains(35));
    /// ```
    #[inline]
    pub fn xor_inplace(&mut self, other: &Self) {
        unsafe { ffi::roaring_bitmap_xor_inplace(self.bitmap, other.bitmap) }
    }

    /// Computes the symmetric difference (xor) between multiple bitmaps
    /// and returns new bitmap as a result.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(15);
    /// bitmap1.add(25);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(25);
    /// bitmap2.add(35);
    ///
    /// let bitmap3 = Bitmap::fast_xor(&[&bitmap1, &bitmap2]);
    ///
    /// assert!(bitmap3.cardinality() == 2);
    /// assert!(bitmap3.contains(15));
    /// assert!(!bitmap3.contains(25));
    /// assert!(bitmap3.contains(35));
    /// ```
    #[inline]
    pub fn fast_xor(bitmaps: &[&Bitmap]) -> Self {
        let mut bms: Vec<*const ffi::roaring_bitmap_s> = Vec::with_capacity(bitmaps.len());

        for (i, item) in bitmaps.iter().enumerate() {
            bms.insert(i, item.bitmap);
        }

        Bitmap {
            bitmap: unsafe { ffi::roaring_bitmap_xor_many(bms.len(), bms.as_mut_ptr()) },
        }
    }

    /// Computes the difference between two bitmaps and returns the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    ///
    /// bitmap1.add(15);
    /// bitmap1.add(25);
    ///
    /// let mut bitmap2 = Bitmap::create();
    ///
    /// bitmap2.add(25);
    /// bitmap2.add(35);
    ///
    /// let bitmap3 = bitmap1.andnot(&bitmap2);
    ///
    /// assert_eq!(bitmap3.cardinality(), 1);
    /// assert!(bitmap3.contains(15));
    /// assert!(!bitmap3.contains(25));
    /// assert!(!bitmap3.contains(35));
    /// ```
    #[inline]
    pub fn andnot(&self, other: &Self) -> Self {
        Bitmap {
            bitmap: unsafe { ffi::roaring_bitmap_andnot(self.bitmap, other.bitmap) },
        }
    }

    /// Computes the difference between two bitmaps and stores the result
    /// in the current bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    ///
    /// bitmap1.add(15);
    /// bitmap1.add(25);
    ///
    /// let mut bitmap2 = Bitmap::create();
    ///
    /// bitmap2.add(25);
    /// bitmap2.add(35);
    ///
    /// bitmap1.andnot_inplace(&bitmap2);
    ///
    /// assert_eq!(bitmap1.cardinality(), 1);
    /// assert!(bitmap1.contains(15));
    /// assert!(!bitmap1.contains(25));
    /// assert!(!bitmap1.contains(35));
    ///
    /// let mut bitmap3 = Bitmap::create();
    /// bitmap3.add(15);
    /// let mut bitmap4 = Bitmap::create();
    /// bitmap3.andnot_inplace(&bitmap4);
    /// assert_eq!(bitmap3.cardinality(), 1);
    /// assert!(bitmap3.contains(15));
    /// ```
    #[inline]
    pub fn andnot_inplace(&mut self, other: &Self) {
        unsafe { ffi::roaring_bitmap_andnot_inplace(self.bitmap, other.bitmap) }
    }

    /// Negates the bits in the given range (i.e., [rangeStart..rangeEnd)),
    /// any integer present in this range and in the bitmap is removed.
    /// Returns result as a new bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(4);
    ///
    /// let bitmap2 = bitmap1.flip((1..3));
    ///
    /// assert_eq!(bitmap2.cardinality(), 3);
    /// assert!(bitmap2.contains(1));
    /// assert!(bitmap2.contains(2));
    /// assert!(!bitmap2.contains(3));
    /// assert!(bitmap2.contains(4));
    /// ```
    #[inline]
    pub fn flip(&self, range: Range<u64>) -> Self {
        Bitmap {
            bitmap: unsafe { ffi::roaring_bitmap_flip(self.bitmap, range.start, range.end) },
        }
    }

    /// Negates the bits in the given range (i.e., [rangeStart..rangeEnd)),
    /// any integer present in this range and in the bitmap is removed.
    /// Stores the result in the current bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(4);
    /// bitmap1.flip_inplace((1..3));
    ///
    /// assert_eq!(bitmap1.cardinality(), 3);
    /// assert!(bitmap1.contains(1));
    /// assert!(bitmap1.contains(2));
    /// assert!(!bitmap1.contains(3));
    /// assert!(bitmap1.contains(4));
    /// ```
    #[inline]
    pub fn flip_inplace(&mut self, range: Range<u64>) {
        unsafe { ffi::roaring_bitmap_flip_inplace(self.bitmap, range.start, range.end) }
    }

    /// Returns a vector containing all of the integers stored in the Bitmap
    /// in sorted order.
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::create();
    /// bitmap.add(15);
    /// bitmap.add(25);
    ///
    /// assert_eq!(bitmap.to_vec(), [15, 25]);
    /// assert!(bitmap.to_vec() != [10, 15, 25]);
    /// ```
    #[inline]
    pub fn to_vec(&self) -> Vec<u32> {
        let bitmap_size = self.cardinality();

        let mut buffer: Vec<u32> = Vec::with_capacity(bitmap_size as usize);
        let buffer_ptr = buffer.as_mut_ptr();

        unsafe {
            ::std::mem::forget(buffer);

            ffi::roaring_bitmap_to_uint32_array(self.bitmap, buffer_ptr);

            Vec::from_raw_parts(buffer_ptr, bitmap_size as usize, bitmap_size as usize)
        }
    }

    /// Computes the serialized size in bytes of the Bitmap.
    #[inline]
    pub fn get_serialized_size_in_bytes(&self) -> usize {
        unsafe { ffi::roaring_bitmap_portable_size_in_bytes(self.bitmap) }
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
    pub fn serialize(&self) -> Vec<u8> {
        let capacity = self.get_serialized_size_in_bytes();
        let mut dst = Vec::with_capacity(capacity);

        unsafe {
            ffi::roaring_bitmap_portable_serialize(
                self.bitmap,
                dst.as_mut_ptr() as *mut ::libc::c_char,
            );
            dst.set_len(capacity);
        }

        dst
    }

    /// Given a serialized bitmap as slice of bytes returns a bitmap instance.
    /// See example of #serialize function.
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
    pub fn try_deserialize(buffer: &[u8]) -> Option<Self> {
        unsafe {
            let bitmap = ffi::roaring_bitmap_portable_deserialize_safe(
                buffer.as_ptr() as *const ::libc::c_char,
                buffer.len() as ::libc::size_t
            );

            if !bitmap.is_null() {
                Some(Bitmap { bitmap })
            } else {
                None
            }
        }
    }

    /// Given a serialized bitmap as slice of bytes returns a bitmap instance.
    /// See example of #serialize function.
    ///
    /// On invalid input returns empty bitmap.
    #[inline]
    pub fn deserialize(buffer: &[u8]) -> Self {
        Self::try_deserialize(buffer).unwrap_or(Bitmap::create())
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
    pub fn of(elements: &[u32]) -> Self {
        Bitmap {
            bitmap: unsafe { ffi::roaring_bitmap_of_ptr(elements.len(), elements.as_ptr()) },
        }
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
    /// assert!(bitmap.run_optimize());
    /// ```
    #[inline]
    pub fn run_optimize(&mut self) -> bool {
        unsafe { ffi::roaring_bitmap_run_optimize(self.bitmap) }
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
    pub fn remove_run_compression(&mut self) -> bool {
        unsafe { ffi::roaring_bitmap_remove_run_compression(self.bitmap) }
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
    pub fn is_empty(&self) -> bool {
        unsafe { ffi::roaring_bitmap_is_empty(self.bitmap) }
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
    pub fn is_subset(&self, other: &Self) -> bool {
        unsafe { ffi::roaring_bitmap_is_subset(self.bitmap, other.bitmap) }
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
    pub fn is_strict_subset(&self, other: &Self) -> bool {
        unsafe { ffi::roaring_bitmap_is_strict_subset(self.bitmap, other.bitmap) }
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
    pub fn intersect(&self, other: &Self) -> bool {
        unsafe { ffi::roaring_bitmap_intersect(self.bitmap, other.bitmap) }
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
    pub fn jaccard_index(&self, other: &Self) -> f64 {
        unsafe { ffi::roaring_bitmap_jaccard_index(self.bitmap, other.bitmap) }
    }

    /// Return the size of the intersection between Self and &other
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(1);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(1);
    /// bitmap2.add(2);
    ///
    /// assert_eq!(bitmap1.and_cardinality(&bitmap2), 1);
    /// ```
    #[inline]
    pub fn and_cardinality(&self, other: &Self) -> u64 {
        unsafe { ffi::roaring_bitmap_and_cardinality(self.bitmap, other.bitmap) }
    }

    /// Return the size of the union between Self and &other
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(15);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(25);
    ///
    /// assert_eq!(bitmap1.or_cardinality(&bitmap2), 2);
    #[inline]
    pub fn or_cardinality(&self, other: &Self) -> u64 {
        unsafe { ffi::roaring_bitmap_or_cardinality(self.bitmap, other.bitmap) }
    }

    /// Return the size of the difference between Self and &other
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    ///
    /// bitmap1.add(15);
    /// bitmap1.add(25);
    ///
    /// let mut bitmap2 = Bitmap::create();
    ///
    /// bitmap2.add(25);
    /// bitmap2.add(35);
    ///
    /// assert_eq!(bitmap1.andnot_cardinality(&bitmap2), 1);
    /// ```
    #[inline]
    pub fn andnot_cardinality(&self, other: &Self) -> u64 {
        unsafe { ffi::roaring_bitmap_andnot_cardinality(self.bitmap, other.bitmap) }
    }

    /// Return the size of the symmetric difference between Self and &other
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(15);
    /// bitmap1.add(25);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(25);
    /// bitmap2.add(35);
    ///
    /// assert_eq!(bitmap1.xor_cardinality(&bitmap2), 2);
    /// ```
    #[inline]
    pub fn xor_cardinality(&self, other: &Self) -> u64 {
        unsafe { ffi::roaring_bitmap_xor_cardinality(self.bitmap, other.bitmap) }
    }

    /// Returns the smallest value in the set.
    /// Returns std::u32::MAX if the set is empty.
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
    pub fn minimum(&self) -> Option<u32> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { ffi::roaring_bitmap_minimum(self.bitmap) })
        }
    }

    /// Returns the greatest value in the set.
    /// Returns 0 if the set is empty.
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
    pub fn maximum(&self) -> Option<u32> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { ffi::roaring_bitmap_maximum(self.bitmap) })
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
    /// ```
    #[inline]
    pub fn rank(&self, x: u32) -> u64 {
        unsafe { ffi::roaring_bitmap_rank(self.bitmap, x) }
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
    pub fn select(&self, rank: u32) -> Option<u32> {
        let mut element: u32 = 0;
        let result = unsafe { ffi::roaring_bitmap_select(self.bitmap, rank, &mut element) };

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
    pub fn statistics(&self) -> Statistics {
        let mut statistics: ffi::roaring_statistics_s = unsafe { ::std::mem::zeroed() };

        unsafe { ffi::roaring_bitmap_statistics(self.bitmap, &mut statistics) };

        statistics
    }
}
