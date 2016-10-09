use std::slice;
use std::ops::Range;

use {Bitmap, Statistics, ffi};

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

        Bitmap { bitmap: bitmap }
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

        Bitmap { bitmap: bitmap }
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
    pub fn add_many(&mut self, elements: &[u32]) -> () {
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
    pub fn add(&mut self, element: u32) -> () {
        unsafe { ffi::roaring_bitmap_add(self.bitmap, element) }
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
    /// bitmap.remove(1);
    ///
    /// assert!(bitmap.is_empty());
    /// ```
    #[inline]
    pub fn remove(&mut self, element: u32) -> () {
        unsafe { ffi::roaring_bitmap_remove(self.bitmap, element) }
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
        Bitmap { bitmap: unsafe { ffi::roaring_bitmap_and(self.bitmap, other.bitmap) } }
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
    pub fn and_inplace(&mut self, other: &Self) -> () {
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
        Bitmap { bitmap: unsafe { ffi::roaring_bitmap_or(self.bitmap, other.bitmap) } }
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
    pub fn or_inplace(&mut self, other: &Self) -> () {
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
            bitmap: unsafe { ffi::roaring_bitmap_or_many(bms.len(), bms.as_mut_ptr()) }
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
            bitmap: unsafe { ffi::roaring_bitmap_or_many_heap(bms.len() as u32, bms.as_mut_ptr()) }
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
        Bitmap { bitmap: unsafe { ffi::roaring_bitmap_xor(self.bitmap, other.bitmap) } }
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
    pub fn xor_inplace(&mut self, other: &Self) -> () {
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
            bitmap: unsafe { ffi::roaring_bitmap_xor_many(bms.len(), bms.as_mut_ptr()) }
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
        Bitmap { bitmap: unsafe { ffi::roaring_bitmap_andnot(self.bitmap, other.bitmap) } }
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
    /// ```
    #[inline]
    pub fn andnot_inplace(&mut self, other: &Self) -> () {
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
        Bitmap { bitmap: unsafe { ffi::roaring_bitmap_flip(self.bitmap, range.start, range.end) } }
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
    pub fn flip_inplace(&mut self, range: Range<u64>) -> () {
        unsafe { ffi::roaring_bitmap_flip_inplace(self.bitmap, range.start, range.end) }
    }

    /// Creates a new slice containing all of the integers stored in the Bitmap
    /// in sorted order.
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::create();
    /// bitmap.add(15);
    /// bitmap.add(25);
    ///
    /// assert_eq!(bitmap.as_slice(), [15, 25]);
    /// assert!(bitmap.as_slice() != [10, 15, 25])
    /// ```
    #[inline]
    pub fn as_slice(&self) -> &[u32] {
        let bitmap_size = self.cardinality();

        let mut buffer: Vec<u32> = Vec::with_capacity(bitmap_size as usize);

        unsafe {
            ffi::roaring_bitmap_to_uint32_array(self.bitmap, buffer.as_mut_ptr());

            slice::from_raw_parts(buffer.as_ptr(), bitmap_size as usize)
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
        let mut dst = Vec::with_capacity(self.get_serialized_size_in_bytes());

        unsafe {
            ffi::roaring_bitmap_portable_serialize(self.bitmap, dst.as_mut_ptr() as *mut ::libc::c_char);
        }

        dst
    }

    /// Given a serialized bitmap as slice of bytes returns a bitmap instance.
    /// See example of #serialize function.
    #[inline]
    pub fn deserialize(buffer: &[u8]) -> Self {
        unsafe {
            Bitmap  {
                bitmap: ffi::roaring_bitmap_portable_deserialize(buffer.as_ptr() as *const ::libc::c_char)
            }
        }
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
        let mut statistics: ffi::roaring_statistics_s = Default::default();

        unsafe { ffi::roaring_bitmap_statistics(self.bitmap, &mut statistics) };

        statistics
    }
}
