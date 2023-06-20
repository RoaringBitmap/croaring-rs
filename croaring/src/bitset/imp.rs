use super::Bitset;
use std::{mem, ptr};

impl Bitset {
    #[inline]
    #[allow(clippy::assertions_on_constants)]
    pub(super) unsafe fn take_heap(p: *mut ffi::bitset_t) -> Self {
        assert!(!p.is_null());
        let result = Self { bitset: p.read() };
        // It seems unlikely that the bitset type will meaningfully change, but check if we ever go
        // to a version 2.
        const _: () = assert!(ffi::ROARING_VERSION_MAJOR == 1);
        ffi::roaring_free(p.cast());
        result
    }

    /// Access the raw underlying slice
    #[inline]
    pub const fn as_slice(&self) -> &[u64] {
        if self.bitset.arraysize == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.bitset.array, self.bitset.arraysize) }
        }
    }

    /// Access the raw underlying slice
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u64] {
        if self.bitset.arraysize == 0 {
            &mut []
        } else {
            unsafe { std::slice::from_raw_parts_mut(self.bitset.array, self.bitset.arraysize) }
        }
    }

    /// Create a new bitset
    ///
    /// Does not allocate
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitset;
    /// let bitset = Bitset::new();
    /// assert_eq!(bitset.capacity(), 0);
    /// assert_eq!(bitset.size_in_bits(), 0);
    /// ```
    #[inline]
    #[doc(alias = "bitset_create")]
    pub const fn new() -> Self {
        Self {
            bitset: ffi::bitset_t {
                array: ptr::null_mut(),
                arraysize: 0,
                capacity: 0,
            },
        }
    }

    /// Create a new bitset of the specified size
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitset;
    /// let bitset = Bitset::with_size(100);
    /// // Actual size/capacity may be rounded up
    /// assert!(bitset.capacity() >= 100);
    /// assert!(bitset.size_in_bits() >= 100);
    /// ```
    #[inline]
    #[doc(alias = "bitset_create_with_capacity")]
    pub fn with_size(size: usize) -> Self {
        unsafe { Self::take_heap(ffi::bitset_create_with_capacity(size)) }
    }

    /// Capacity in bits
    #[inline]
    pub const fn capacity(&self) -> usize {
        self.bitset.capacity * 64
    }

    /// Set all bits to zero
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitset;
    /// let mut bitset = Bitset::with_size(64);
    /// bitset.fill(true);
    /// assert!(bitset.get(1));
    /// assert!(bitset.get(63));
    /// // Bitset size stays the same
    /// assert!(!bitset.get(64));
    /// bitset.fill(false);
    /// assert!(!bitset.get(1));
    /// assert!(!bitset.get(63));
    /// assert!(!bitset.get(64));
    /// ```
    #[inline]
    #[doc(alias = "bitset_clear")]
    #[doc(alias = "bitset_fill")]
    pub fn fill(&mut self, value: bool) {
        if value {
            unsafe { ffi::bitset_fill(&mut self.bitset) };
        } else {
            unsafe { ffi::bitset_clear(&mut self.bitset) };
        }
    }

    /// How many bytes of memory the backend buffer uses
    #[inline]
    #[doc(alias = "bitset_size_in_bytes")]
    pub const fn size_in_bytes(&self) -> usize {
        self.size_in_words() * mem::size_of::<u64>()
    }

    /// How many bits can be accessed
    #[inline]
    #[doc(alias = "bitset_size_in_bits")]
    pub const fn size_in_bits(&self) -> usize {
        self.size_in_bytes() * 8
    }

    /// How many 64-bit words of memory the backend buffer uses
    #[inline]
    #[doc(alias = "bitset_size_in_words")]
    pub const fn size_in_words(&self) -> usize {
        self.bitset.arraysize
    }

    /// Resize the bitset to contain `new_array_size` 64-bit words
    ///
    /// New bits are set to `value`
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitset;
    /// let mut bitset = Bitset::new();
    /// bitset.resize_words(1, false);
    /// bitset.resize_words(2, true);
    /// assert_eq!(bitset.iter().collect::<Vec<_>>(), (64..128).collect::<Vec<_>>());
    pub fn resize_words(&mut self, new_array_size: usize, value: bool) {
        let old_array_size = self.bitset.arraysize;
        let res = unsafe { ffi::bitset_resize(&mut self.bitset, new_array_size, value) };
        assert!(res);
        if new_array_size > old_array_size {
            let new_data_slice = &mut self.as_mut_slice()[old_array_size..];
            new_data_slice.fill((value as u64) * !0);
        }
    }

    /// For advanced users: Grow the bitset so that it can support newarraysize * 64 bits with padding.
    /// Return true in case of success, false for failure
    #[inline]
    #[doc(alias = "bitset_grow")]
    fn grow(&mut self, new_array_size: usize) {
        assert!(unsafe { ffi::bitset_grow(&mut self.bitset, new_array_size) });
    }

    /// Attempts to recover unused memory by shrinking capacity to fit the highest set bit
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitset;
    /// let mut bitset = Bitset::new();
    /// bitset.set(63);
    /// bitset.set(1000);
    /// assert!(bitset.size_in_bits() > 1000);
    /// bitset.set_to_value(1000, false);
    /// bitset.shrink_to_fit();
    /// // The next highest bit is at index 63
    /// assert_eq!(bitset.size_in_bits(), 64);
    #[inline]
    #[doc(alias = "bitset_trim")]
    pub fn shrink_to_fit(&mut self) {
        unsafe { ffi::bitset_trim(&mut self.bitset) };
    }

    /// Set the ith bit
    ///
    /// Will resize the bitset if needed, any other newly added bits will be initialized to zero
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitset;
    /// let mut bitset = Bitset::new();
    /// bitset.set(1);
    /// bitset.set(2);
    /// bitset.set(100);
    /// assert_eq!(bitset.iter().collect::<Vec<_>>(), vec![1, 2, 100]);
    /// ```
    #[inline]
    #[doc(alias = "bitset_set")]
    pub fn set(&mut self, i: usize) {
        let array_idx = i / 64;
        if array_idx >= self.bitset.arraysize {
            self.grow(array_idx + 1);
        }
        self.as_mut_slice()[array_idx] |= 1 << (i % 64);
    }

    /// Set the ith bit to `value`
    ///
    /// Will resize the bitset if needed, any other newly added bits will be initialized to zero
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitset;
    /// let mut bitset = Bitset::new();
    /// bitset.set_to_value(1, true);
    /// bitset.set_to_value(2, true);
    /// bitset.set_to_value(100, true);
    /// bitset.set_to_value(1, false);
    /// assert_eq!(bitset.iter().collect::<Vec<_>>(), vec![2, 100]);
    /// ```
    #[inline]
    #[doc(alias = "bitset_set_to_value")]
    pub fn set_to_value(&mut self, i: usize, value: bool) {
        let array_idx = i / 64;
        if array_idx >= self.bitset.arraysize {
            self.grow(array_idx + 1);
        }
        let dst = &mut self.as_mut_slice()[array_idx];
        let mask = 1 << (i % 64);
        let value_bit = (value as u64) << (i % 64);
        let mut word = *dst;
        word &= !mask;
        word |= value_bit;
        *dst = word;
    }

    /// Get the value of the ith bit
    ///
    /// If the bit is out of bounds, returns false
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitset;
    /// let mut bitset = Bitset::new();
    /// bitset.set(1);
    /// bitset.set(2);
    /// assert!(bitset.get(1));
    /// assert!(bitset.get(2));
    /// assert!(!bitset.get(3));
    /// ```
    #[inline]
    #[doc(alias = "bitset_get")]
    pub const fn get(&self, i: usize) -> bool {
        let array_idx = i / 64;
        if array_idx >= self.bitset.arraysize {
            return false;
        }
        let word = self.as_slice()[array_idx];
        let mask = 1 << (i % 64);
        (word & mask) != 0
    }

    /// Count of number of set bits
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitset;
    /// let mut bitset: Bitset = [1, 2, 3, 100].into_iter().collect();
    /// assert_eq!(bitset.count(), 4);
    /// ```
    #[inline]
    #[doc(alias = "bitset_count")]
    pub fn count(&self) -> usize {
        unsafe { ffi::bitset_count(&self.bitset) }
    }

    /// Index of the first set bit, or zero if the bitset has no set bits
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitset;
    /// let mut bitset = Bitset::new();
    /// // minimum returns 0 if the bitset is empty
    /// assert_eq!(bitset.minimum(), 0);
    /// bitset.set(100);
    /// assert_eq!(bitset.minimum(), 100);
    /// ```
    #[inline]
    #[doc(alias = "bitset_minimum")]
    pub fn minimum(&self) -> usize {
        unsafe { ffi::bitset_minimum(&self.bitset) }
    }

    /// Index of the last set bit, or zero if the bitset has no set bits
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitset;
    /// let mut bitset = Bitset::new();
    /// // maximum returns 0 if the bitset is empty
    /// assert_eq!(bitset.maximum(), 0);
    /// bitset.set(100);
    /// assert_eq!(bitset.maximum(), 100);
    /// bitset.set(1000);
    /// assert_eq!(bitset.maximum(), 1000);
    /// ```
    #[inline]
    #[doc(alias = "bitset_maximum")]
    pub fn maximum(&self) -> usize {
        unsafe { ffi::bitset_maximum(&self.bitset) }
    }

    /// The size of the hypothetical union of `self` and `other`
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitset;
    /// let mut bitset1: Bitset = [1, 2, 3, 100].into_iter().collect();
    /// let bitset2: Bitset = [2, 3, 4, 5].into_iter().collect();
    /// assert_eq!(bitset1.union_count(&bitset2), 6);
    /// bitset1 |= &bitset2;
    /// assert_eq!(bitset1.count(), 6);
    /// ```
    #[inline]
    #[doc(alias = "bitset_union_count")]
    pub fn union_count(&self, other: &Self) -> usize {
        // CRoaring uses restrict pointers, so we can't use the same pointer for both
        if ptr::eq(self, other) {
            return self.count();
        }
        unsafe { ffi::bitset_union_count(&self.bitset, &other.bitset) }
    }

    /// The size of the hypothetical intersection of `self` and `other`
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitset;
    /// let mut bitset1: Bitset = [1, 2, 3, 100].into_iter().collect();
    /// let bitset2: Bitset = [2, 3, 4, 5].into_iter().collect();
    /// assert_eq!(bitset1.intersection_count(&bitset2), 2);
    /// bitset1 &= &bitset2;
    /// assert_eq!(bitset1.count(), 2);
    /// ```
    #[inline]
    #[doc(alias = "bitset_intersection_count")]
    pub fn intersection_count(&self, other: &Self) -> usize {
        // CRoaring uses restrict pointers, so we can't use the same pointer for both
        if ptr::eq(self, other) {
            return self.count();
        }
        unsafe { ffi::bitset_intersection_count(&self.bitset, &other.bitset) }
    }

    /// Return true if `self` and `other` contain no common elements
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitset;
    /// let bitset1: Bitset = [1, 2, 3, 100].into_iter().collect();
    /// let bitset2: Bitset = [2, 3, 4, 5].into_iter().collect();
    /// assert!(!bitset1.is_disjoint(&bitset2));
    /// let bitset3: Bitset = [4, 5, 6, 7].into_iter().collect();
    /// assert!(bitset1.is_disjoint(&bitset3));
    /// ```
    ///
    /// Empty bitsets are always disjoint
    ///
    /// ```
    /// use croaring::Bitset;
    /// let bitset1 = Bitset::new();
    /// let bitset2 = Bitset::new();
    /// assert!(bitset1.is_disjoint(&bitset2));
    /// ```
    #[inline]
    #[doc(alias = "bitset_disjoint")]
    pub fn is_disjoint(&self, other: &Self) -> bool {
        // CRoaring uses restrict pointers, so we can't use the same pointer for both
        if ptr::eq(self, other) {
            return false;
        }
        unsafe { ffi::bitsets_disjoint(&self.bitset, &other.bitset) }
    }

    /// Return true if `self` and `other` contain at least one common element
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitset;
    /// let bitset1: Bitset = [1, 2, 3, 100].into_iter().collect();
    /// let bitset2: Bitset = [2, 3, 4, 5].into_iter().collect();
    /// assert!(bitset1.has_intersect(&bitset2));
    /// let bitset3: Bitset = [4, 5, 6, 7].into_iter().collect();
    /// assert!(!bitset1.has_intersect(&bitset3));
    /// ```
    #[inline]
    #[doc(alias = "bitset_intersect")]
    pub fn has_intersect(&self, other: &Self) -> bool {
        // CRoaring uses restrict pointers, so we can't use the same pointer for both
        if ptr::eq(self, other) {
            return true;
        }
        unsafe { ffi::bitsets_intersect(&self.bitset, &other.bitset) }
    }

    /// Return true if `self` is a superset of `other`
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitset;
    /// let bitset1: Bitset = [1, 2, 3, 100].into_iter().collect();
    /// let bitset2: Bitset = [2, 3].into_iter().collect();
    /// assert!(bitset1.is_superset(&bitset2));
    /// let bitset3: Bitset = [4, 5, 6, 7].into_iter().collect();
    /// assert!(!bitset1.is_superset(&bitset3));
    /// ```
    #[inline]
    #[doc(alias = "bitset_contains_all")]
    pub fn is_superset(&self, other: &Self) -> bool {
        // CRoaring uses restrict pointers, so we can't use the same pointer for both
        if ptr::eq(self, other) {
            return true;
        }
        unsafe { ffi::bitset_contains_all(&self.bitset, &other.bitset) }
    }

    /// The size of the hypothetical difference of `self` and `other`
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitset;
    /// let mut bitset1: Bitset = [1, 2, 3, 100].into_iter().collect();
    /// let bitset2: Bitset = [2, 3, 4, 5].into_iter().collect();
    /// assert_eq!(bitset1.difference_count(&bitset2), 2);
    /// bitset1 -= &bitset2;
    /// assert_eq!(bitset1.count(), 2);
    /// ```
    #[inline]
    #[doc(alias = "bitset_difference_count")]
    pub fn difference_count(&self, other: &Self) -> usize {
        // CRoaring uses restrict pointers, so we can't use the same pointer for both
        if ptr::eq(self, other) {
            return 0;
        }
        unsafe { ffi::bitset_difference_count(&self.bitset, &other.bitset) }
    }

    /// The size of the hypothetical symmetric difference (xor) of `self` and `other`
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitset;
    /// let mut bitset1: Bitset = [1, 2, 3, 100].into_iter().collect();
    /// let bitset2: Bitset = [2, 3, 4, 5].into_iter().collect();
    /// assert_eq!(bitset1.symmetric_difference_count(&bitset2), 4);
    /// bitset1 ^= &bitset2;
    /// assert_eq!(bitset1.count(), 4);
    /// ```
    #[inline]
    #[doc(alias = "bitset_symmetric_difference_count")]
    #[doc(alias = "xor_count")]
    pub fn symmetric_difference_count(&self, other: &Self) -> usize {
        // CRoaring uses restrict pointers, so we can't use the same pointer for both
        if ptr::eq(self, other) {
            return 0;
        }
        unsafe { ffi::bitset_symmetric_difference_count(&self.bitset, &other.bitset) }
    }

    /// Expose the raw CRoaring bitset
    ///
    /// This allows calling raw CRoaring functions on the bitset.
    #[inline]
    pub fn as_raw(&self) -> &ffi::bitset_t {
        &self.bitset
    }

    /// Expose the raw CRoaring bitset mutibly
    ///
    /// This allows calling raw CRoaring functions on the bitset.
    #[inline]
    pub fn as_raw_mut(&mut self) -> &mut ffi::bitset_t {
        &mut self.bitset
    }
}
