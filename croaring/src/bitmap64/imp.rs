use super::{Bitmap64, Deserializer, Serializer, Statistics};
use core::mem::MaybeUninit;
use core::ops::{Bound, RangeBounds};
use core::prelude::v1::*;
use core::ptr::{self, NonNull};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

impl Bitmap64 {
    #[inline]
    pub(crate) unsafe fn take_heap(p: *mut ffi::roaring64_bitmap_t) -> Self {
        let raw = NonNull::new(p).expect("non-null ptr");
        Self { raw }
    }

    /// Create a new empty bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let bitmap = Bitmap64::new();
    /// assert_eq!(bitmap.cardinality(), 0);
    /// ```
    #[inline]
    #[must_use]
    #[doc(alias = "roaring64_bitmap_create")]
    pub fn new() -> Self {
        unsafe { Self::take_heap(ffi::roaring64_bitmap_create()) }
    }

    /// Creates a new bitmap from a slice of u64 integers
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let bitmap = Bitmap64::of(&[1, 2, 3]);
    /// assert_eq!(bitmap.cardinality(), 3);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_of_ptr")]
    #[must_use]
    pub fn of(slice: &[u64]) -> Self {
        unsafe { Self::take_heap(ffi::roaring64_bitmap_of_ptr(slice.len(), slice.as_ptr())) }
    }

    /// Create a new bitmap containing all the values in a range
    #[inline]
    #[doc(alias = "roaring64_bitmap_from_range")]
    #[must_use]
    pub fn from_range<R: RangeBounds<u64>>(range: R) -> Self {
        Self::from_range_with_step(range, 1)
    }

    /// Create a new bitmap containing all the values in `range` which are a multiple of `step` away from the lower
    /// bound
    ///
    /// If `step` is 0 or there are no values which are a multiple of `step` away from the lower bound within range,
    /// an empty bitmap is returned
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ops::Bound;
    /// use croaring::Bitmap64;
    /// let bitmap = Bitmap64::from_range_with_step(0..10, 3);
    /// assert_eq!(bitmap.iter().collect::<Vec<_>>(), vec![0, 3, 6, 9]);
    ///
    /// // empty ranges
    /// assert_eq!(Bitmap64::from_range_with_step(0..0, 1), Bitmap64::new());
    /// assert_eq!(Bitmap64::from_range_with_step(100..=0, 0), Bitmap64::new());
    ///
    /// // step is 0
    /// assert_eq!(Bitmap64::from_range_with_step(0..10, 0), Bitmap64::new());
    ///
    /// // No values of step in range
    /// let bitmap = Bitmap64::from_range_with_step((Bound::Excluded(0), Bound::Included(10)), 100);
    /// assert_eq!(bitmap, Bitmap64::new());
    /// let bitmap = Bitmap64::from_range_with_step((Bound::Excluded(u64::MAX), Bound::Included(u64::MAX)), 1);
    /// assert_eq!(bitmap, Bitmap64::new());
    ///
    /// // Exclusive ranges still step from the start, but do not include it
    /// let bitmap = Bitmap64::from_range_with_step((Bound::Excluded(10), Bound::Included(30)), 10);
    /// assert_eq!(bitmap.iter().collect::<Vec<_>>(), vec![20, 30]);
    ///
    /// // Ranges including max value
    /// let bitmap = Bitmap64::from_range_with_step((u64::MAX - 1)..=u64::MAX, 1);
    /// assert_eq!(bitmap.iter().collect::<Vec<_>>(), vec![u64::MAX - 1, u64::MAX]);
    /// let bitmap = Bitmap64::from_range_with_step((u64::MAX - 1)..=u64::MAX, 3);
    /// assert_eq!(bitmap.iter().collect::<Vec<_>>(), vec![u64::MAX - 1]);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_from_range")]
    #[must_use]
    pub fn from_range_with_step<R: RangeBounds<u64>>(range: R, step: u64) -> Self {
        // This can't use `range_to_exclusive` because when the start is excluded, we want
        // to start at the next step, not one more
        let start = match range.start_bound() {
            Bound::Included(&i) => i,
            Bound::Excluded(&i) => match i.checked_add(step) {
                Some(i) => i,
                None => return Self::new(),
            },
            Bound::Unbounded => 0,
        };
        let end_inclusive = match range.end_bound() {
            Bound::Included(&i) => i,
            Bound::Excluded(&i) => match i.checked_sub(1) {
                Some(i) => i,
                None => return Self::new(),
            },
            Bound::Unbounded => u64::MAX,
        };
        // roaring64_bitmap_from_range takes an exclusive range,
        // so we need to handle the case where the range should include u64::MAX,
        // and manually add it in afterwards since there's no way to set it with an exclusive range
        let (end, add_max) = match end_inclusive.checked_add(1) {
            Some(i) => (i, false),
            None => (u64::MAX, (u64::MAX - start) % step == 0),
        };

        unsafe {
            let result = ffi::roaring64_bitmap_from_range(start, end, step);
            if result.is_null() {
                Self::new()
            } else {
                let mut result = Self::take_heap(result);
                if add_max {
                    result.add(u64::MAX);
                }
                result
            }
        }
    }

    /// Add a value to the bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap = Bitmap64::new();
    /// bitmap.add(1);
    /// assert!(bitmap.contains(1));
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_add")]
    pub fn add(&mut self, value: u64) {
        unsafe { ffi::roaring64_bitmap_add(self.raw.as_ptr(), value) }
    }

    /// Add the integer element to the bitmap. Returns true if the value was
    /// added, false if the value was already in the bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// let mut bitmap = Bitmap64::new();
    /// assert!(bitmap.add_checked(1));
    /// assert!(!bitmap.add_checked(1));
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_add_checked")]
    pub fn add_checked(&mut self, value: u64) -> bool {
        unsafe { ffi::roaring64_bitmap_add_checked(self.raw.as_ptr(), value) }
    }

    /// Add many values to the bitmap
    ///
    /// See also [`Bitmap64::extend`]
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// let mut bitmap = Bitmap64::new();
    /// bitmap.add_many(&[1, 2, 3]);
    ///
    /// assert!(bitmap.contains(1));
    /// assert!(bitmap.contains(2));
    /// assert!(bitmap.contains(3));
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_add_many")]
    pub fn add_many(&mut self, values: &[u64]) {
        unsafe { ffi::roaring64_bitmap_add_many(self.raw.as_ptr(), values.len(), values.as_ptr()) }
    }

    /// Add all values in range
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// let mut bitmap = Bitmap64::new();
    /// bitmap.add_range(1..3);
    ///
    /// assert!(bitmap.contains(1));
    /// assert!(bitmap.contains(2));
    /// assert!(!bitmap.contains(3));
    ///
    /// let mut bitmap2 = Bitmap64::new();
    /// bitmap2.add_range(3..1);
    /// assert!(bitmap2.is_empty());
    ///
    /// let mut bitmap3 = Bitmap64::new();
    /// bitmap3.add_range(3..3);
    /// assert!(bitmap3.is_empty());
    ///
    /// let mut bitmap4 = Bitmap64::new();
    /// bitmap4.add_range(..=2);
    /// bitmap4.add_range(u64::MAX..=u64::MAX);
    /// assert!(bitmap4.contains(0));
    /// assert!(bitmap4.contains(1));
    /// assert!(bitmap4.contains(2));
    /// assert!(bitmap4.contains(u64::MAX));
    /// assert_eq!(bitmap4.cardinality(), 4);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_add_range_closed")]
    pub fn add_range<R: RangeBounds<u64>>(&mut self, range: R) {
        let (start, end) = range_to_inclusive(range);
        unsafe { ffi::roaring64_bitmap_add_range_closed(self.raw.as_ptr(), start, end) }
    }

    /// Remove a value from the bitmap if present
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap = Bitmap64::of(&[1, 2, 3]);
    /// bitmap.remove(2);
    /// assert!(!bitmap.contains(2));
    /// bitmap.remove(99); // It is not an error to remove a value not in the bitmap
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_remove")]
    pub fn remove(&mut self, value: u64) {
        unsafe { ffi::roaring64_bitmap_remove(self.raw.as_ptr(), value) }
    }

    /// Remove a value from the bit map if present, and return if the value was previously present
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap = Bitmap64::of(&[1, 2, 3]);
    /// assert!(bitmap.remove_checked(2));
    /// assert!(!bitmap.remove_checked(2));
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_remove_checked")]
    pub fn remove_checked(&mut self, value: u64) -> bool {
        unsafe { ffi::roaring64_bitmap_remove_checked(self.raw.as_ptr(), value) }
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
    /// use croaring::Bitmap64;
    /// let mut bitmap = Bitmap64::of(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    /// bitmap.remove_many(&[1, 2, 3, 4, 5, 6, 7, 8]);
    /// assert_eq!(bitmap.iter().collect::<Vec<_>>(), vec![9]);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_remove_many")]
    pub fn remove_many(&mut self, vals: &[u64]) {
        unsafe { ffi::roaring64_bitmap_remove_many(self.raw.as_ptr(), vals.len(), vals.as_ptr()) }
    }

    /// Remove all values from the specified iterator
    ///
    /// This should be faster than calling `remove` multiple times.
    ///
    /// In order to exploit this optimization, the caller should attempt to keep values with the same high 48 bits of
    /// the value as consecutive elements in `it`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap = Bitmap64::of(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    /// bitmap.remove_all(1..=8); // Remove all values from iterator
    /// assert_eq!(bitmap.iter().collect::<Vec<_>>(), vec![9]);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_remove_bulk")]
    pub fn remove_all<It>(&mut self, it: It)
    where
        It: IntoIterator<Item = u64>,
    {
        let mut ctx = MaybeUninit::<ffi::roaring64_bulk_context_t>::zeroed();
        it.into_iter().for_each(|value| unsafe {
            ffi::roaring64_bitmap_remove_bulk(self.raw.as_ptr(), ctx.as_mut_ptr(), value);
        });
    }

    /// Remove all values in range
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap = Bitmap64::new();
    /// bitmap.add_range(1..4);
    /// assert_eq!(bitmap.iter().collect::<Vec<_>>(), vec![1, 2, 3]);
    ///
    /// bitmap.remove_range(1..=2);
    /// assert_eq!(bitmap.iter().collect::<Vec<_>>(), vec![3]);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_remove_range_closed")]
    pub fn remove_range<R: RangeBounds<u64>>(&mut self, range: R) {
        let (start, end) = range_to_inclusive(range);
        unsafe { ffi::roaring64_bitmap_remove_range_closed(self.raw.as_ptr(), start, end) }
    }

    /// Empty the bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// let mut bitmap = Bitmap64::from([1, 2, 3]);
    /// assert!(!bitmap.is_empty());
    /// bitmap.clear();
    /// assert!(bitmap.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.remove_range(..);
    }

    /// Returns the number of values in the bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap = Bitmap64::new();
    /// assert_eq!(bitmap.cardinality(), 0);
    /// bitmap.add(1);
    /// assert_eq!(bitmap.cardinality(), 1);
    /// ```
    #[inline]
    #[must_use]
    #[doc(alias = "roaring64_bitmap_get_cardinality")]
    pub fn cardinality(&self) -> u64 {
        unsafe { ffi::roaring64_bitmap_get_cardinality(self.raw.as_ptr()) }
    }

    /// Returns the number of values in the bitmap in the given `range`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap = Bitmap64::of(&[1, 3, 4, u64::MAX]);
    ///
    /// assert_eq!(bitmap.range_cardinality(..1), 0);
    /// assert_eq!(bitmap.range_cardinality(..2), 1);
    /// assert_eq!(bitmap.range_cardinality(2..5), 2);
    /// assert_eq!(bitmap.range_cardinality(..5), 3);
    /// assert_eq!(bitmap.range_cardinality(1..=4), 3);
    ///
    /// assert_eq!(bitmap.range_cardinality(4..=u64::MAX), 2);
    /// ```
    #[inline]
    #[must_use]
    #[doc(alias = "roaring64_bitmap_range_cardinality")]
    pub fn range_cardinality<R: RangeBounds<u64>>(&self, range: R) -> u64 {
        let Some(exclusive_range) = range_to_exclusive(range) else {
            return 0;
        };
        self._range_cardinality(exclusive_range)
    }

    #[inline]
    fn _range_cardinality(&self, exclusive_range: ExclusiveRangeRes) -> u64 {
        let ExclusiveRangeRes {
            start,
            end,
            needs_max,
        } = exclusive_range;
        let mut cardinality =
            unsafe { ffi::roaring64_bitmap_range_cardinality(self.raw.as_ptr(), start, end) };
        if needs_max {
            cardinality += u64::from(self.contains(u64::MAX));
        }
        cardinality
    }

    /// Returns true if the bitmap is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap = Bitmap64::new();
    /// assert!(bitmap.is_empty());
    /// bitmap.add(1);
    /// assert!(!bitmap.is_empty());
    /// ```
    #[inline]
    #[must_use]
    #[doc(alias = "roaring64_bitmap_is_empty")]
    pub fn is_empty(&self) -> bool {
        unsafe { ffi::roaring64_bitmap_is_empty(self.raw.as_ptr()) }
    }

    /// Returns true if all the elements of self are in other
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// let bitmap1: Bitmap64 = (5..10).collect();
    /// let bitmap2: Bitmap64 = (5..8).collect();
    /// let bitmap3: Bitmap64 = (5..10).collect();
    /// let bitmap4: Bitmap64 = (9..11).collect();
    ///
    /// assert!(bitmap2.is_subset(&bitmap1));
    /// assert!(bitmap3.is_subset(&bitmap1));
    /// assert!(!bitmap4.is_subset(&bitmap1));
    /// ```
    #[inline]
    #[must_use]
    #[doc(alias = "roaring64_bitmap_is_subset")]
    pub fn is_subset(&self, other: &Self) -> bool {
        unsafe { ffi::roaring64_bitmap_is_subset(self.raw.as_ptr(), other.raw.as_ptr()) }
    }

    /// Returns true if all the elements of self are in other and self is not equal to other
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// let bitmap1: Bitmap64 = (5..9).collect();
    /// let bitmap2: Bitmap64 = (5..8).collect();
    /// let bitmap3: Bitmap64 = (5..10).collect();
    /// let bitmap4: Bitmap64 = (9..11).collect();
    ///
    /// assert!(bitmap2.is_strict_subset(&bitmap1));
    /// assert!(!bitmap3.is_strict_subset(&bitmap1));
    /// assert!(!bitmap4.is_strict_subset(&bitmap1));
    /// assert!(!bitmap1.is_strict_subset(&bitmap1));
    ///
    #[inline]
    #[must_use]
    #[doc(alias = "roaring64_bitmap_is_strict_subset")]
    pub fn is_strict_subset(&self, other: &Self) -> bool {
        unsafe { ffi::roaring64_bitmap_is_strict_subset(self.raw.as_ptr(), other.raw.as_ptr()) }
    }

    /// Returns the smallest value in the bitmap, or None if the bitmap is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap: Bitmap64 = (5..10).collect();
    /// let empty_bitmap: Bitmap64 = Bitmap64::new();
    ///
    /// assert_eq!(bitmap.minimum(), Some(5));
    /// assert_eq!(empty_bitmap.minimum(), None);
    ///
    /// bitmap.add(3);
    ///
    /// assert_eq!(bitmap.minimum(), Some(3));
    /// ```
    #[inline]
    #[must_use]
    #[doc(alias = "roaring64_bitmap_minimum")]
    pub fn minimum(&self) -> Option<u64> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { ffi::roaring64_bitmap_minimum(self.raw.as_ptr()) })
        }
    }

    /// Returns the largest value in the bitmap, or None if the bitmap is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap: Bitmap64 = (5..10).collect();
    /// let empty_bitmap: Bitmap64 = Bitmap64::new();
    ///
    /// assert_eq!(bitmap.maximum(), Some(9));
    /// assert_eq!(empty_bitmap.maximum(), None);
    ///
    /// bitmap.add(15);
    ///
    /// assert_eq!(bitmap.maximum(), Some(15));
    /// ```
    #[inline]
    #[must_use]
    #[doc(alias = "roaring64_bitmap_maximum")]
    pub fn maximum(&self) -> Option<u64> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { ffi::roaring64_bitmap_maximum(self.raw.as_ptr()) })
        }
    }

    /// Attempt to compress the bitmap by finding runs of consecutive values
    ///
    /// Returns true if the bitmap has at least one run container after optimization
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap: Bitmap64 = (100..1000).collect();
    /// assert_eq!(bitmap.cardinality(), 900);
    /// assert!(bitmap.run_optimize());
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_run_optimize")]
    pub fn run_optimize(&mut self) -> bool {
        unsafe { ffi::roaring64_bitmap_run_optimize(self.raw.as_ptr()) }
    }

    /// Returns true if the element is contained in the bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap = Bitmap64::new();
    /// assert!(!bitmap.contains(1));
    /// bitmap.add(1);
    /// assert!(bitmap.contains(1));
    /// ```
    #[inline]
    #[must_use]
    #[doc(alias = "roaring64_bitmap_contains")]
    pub fn contains(&self, value: u64) -> bool {
        unsafe { ffi::roaring64_bitmap_contains(self.raw.as_ptr(), value) }
    }

    /// Check whether a range of values of range are ALL present
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// let bitmap = Bitmap64::of(&[1, 2, 4]);
    /// assert!(bitmap.contains_range(1..=2));
    /// assert!(!bitmap.contains_range(1..=4));
    ///
    /// let mut bitmap = bitmap.clone();
    /// bitmap.add(u64::MAX - 1);
    /// bitmap.add(u64::MAX);
    /// assert!(bitmap.contains_range((u64::MAX - 1)..=u64::MAX));
    ///
    /// // Empty ranges are always contained
    /// assert!(bitmap.contains_range(10..0));
    /// ```
    #[inline]
    #[must_use]
    #[doc(alias = "roaring64_bitmap_contains_range")]
    pub fn contains_range<R: RangeBounds<u64>>(&self, range: R) -> bool {
        let Some(exclusive_range) = range_to_exclusive(range) else {
            return true;
        };
        self._contains_range(exclusive_range)
    }

    #[inline]
    fn _contains_range(&self, exclusive_range: ExclusiveRangeRes) -> bool {
        let ExclusiveRangeRes {
            start,
            end,
            needs_max,
        } = exclusive_range;

        if needs_max && !self.contains(u64::MAX) {
            return false;
        }
        unsafe { ffi::roaring64_bitmap_contains_range(self.raw.as_ptr(), start, end) }
    }

    /// Selects the element at index 'rank' where the smallest element is at index 0
    ///
    /// If the size of the bitmap is strictly greater than rank, then this function returns the element of the given
    /// rank, otherwise, it returns None
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let bitmap: Bitmap64 = (5..10).collect();
    ///
    /// assert_eq!(bitmap.select(0), Some(5));
    /// assert_eq!(bitmap.select(1), Some(6));
    /// assert_eq!(bitmap.select(2), Some(7));
    /// assert_eq!(bitmap.select(3), Some(8));
    /// assert_eq!(bitmap.select(4), Some(9));
    /// assert_eq!(bitmap.select(5), None);
    /// ```
    #[inline]
    #[must_use]
    #[doc(alias = "roaring64_bitmap_select")]
    pub fn select(&self, rank: u64) -> Option<u64> {
        let mut element = 0u64;
        let has_elem: bool =
            unsafe { ffi::roaring64_bitmap_select(self.raw.as_ptr(), rank, &mut element) };

        has_elem.then_some(element)
    }

    /// Returns the number of integers that are smaller or equal to x
    ///
    /// If x is the first element, this function will return 1. If x is smaller than the smallest element, this
    /// function will return 0
    ///
    /// The indexing convention differs between [`Self::select`] and [`Self::rank`]: [`Self::select`] refers to the
    /// smallest value as having index 0, whereas [`Self::rank`] returns 1 when ranking the smallest value
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap: Bitmap64 = (5..10).collect();
    ///
    /// assert_eq!(bitmap.rank(8), 4);
    ///
    /// assert_eq!(bitmap.rank(11), 5);
    /// assert_eq!(bitmap.rank(15), 5);
    ///
    /// bitmap.add(15);
    ///
    /// assert_eq!(bitmap.rank(11), 5);
    /// assert_eq!(bitmap.rank(15), 6);
    /// ```
    #[inline]
    #[must_use]
    #[doc(alias = "roaring64_bitmap_rank")]
    pub fn rank(&self, value: u64) -> u64 {
        unsafe { ffi::roaring64_bitmap_rank(self.raw.as_ptr(), value) }
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
    ///
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// let mut bitmap = Bitmap64::from_range(5..10);
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
    #[must_use]
    #[doc(alias = "roaring64_bitmap_get_index")]
    #[doc(alias = "index")]
    pub fn position(&self, value: u64) -> Option<u64> {
        let mut index = 0u64;
        let has_index: bool =
            unsafe { ffi::roaring64_bitmap_get_index(self.raw.as_ptr(), value, &mut index) };

        has_index.then_some(index)
    }

    /// Negates the bits in the given range
    /// any integer present in this range and in the bitmap is removed.
    /// Returns result as a new bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// let bitmap1 = Bitmap64::of(&[4]);
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
    /// assert_eq!(bitmap3.iter().collect::<Vec<_>>(), [1, 2, 3, 5])
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_flip")]
    #[doc(alias = "roaring64_bitmap_flip_closed")]
    #[must_use]
    pub fn flip<R: RangeBounds<u64>>(&self, range: R) -> Self {
        let (start, end) = range_to_inclusive(range);
        unsafe {
            Self::take_heap(ffi::roaring64_bitmap_flip_closed(
                self.raw.as_ptr(),
                start,
                end,
            ))
        }
    }

    /// Negates the bits in the given range
    /// any integer present in this range and in the bitmap is removed.
    /// Stores the result in the current bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// let mut bitmap1 = Bitmap64::of(&[4]);
    /// bitmap1.flip_inplace(1..3);
    ///
    /// assert_eq!(bitmap1.cardinality(), 3);
    /// assert!(bitmap1.contains(1));
    /// assert!(bitmap1.contains(2));
    /// assert!(!bitmap1.contains(3));
    /// assert!(bitmap1.contains(4));
    /// bitmap1.flip_inplace(4..=4);
    /// assert_eq!(bitmap1.iter().collect::<Vec<_>>(), [1, 2]);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_flip_inplace")]
    pub fn flip_inplace<R: RangeBounds<u64>>(&mut self, range: R) {
        let (start, end) = range_to_inclusive(range);
        unsafe { ffi::roaring64_bitmap_flip_closed_inplace(self.raw.as_ptr(), start, end) };
    }

    /// Returns a vector containing the values in the bitmap in sorted order
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap = Bitmap64::of(&[1, 2, 3]);
    /// assert_eq!(bitmap.to_vec(), vec![1, 2, 3]);
    /// ```
    #[must_use]
    #[cfg(feature = "alloc")]
    pub fn to_vec(&self) -> Vec<u64> {
        let len = self
            .cardinality()
            .try_into()
            .expect("cardinality must fit in a usize");

        let mut vec = alloc::vec![0; len];
        unsafe { ffi::roaring64_bitmap_to_uint64_array(self.raw.as_ptr(), vec.as_mut_ptr()) };
        vec
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
    /// use croaring::{Bitmap64, Portable};
    ///
    /// let original_bitmap: Bitmap64 = (1..5).collect();
    ///
    /// let serialized_buffer = original_bitmap.serialize::<Portable>();
    ///
    /// let deserialized_bitmap = Bitmap64::deserialize::<Portable>(&serialized_buffer);
    ///
    /// assert_eq!(original_bitmap, deserialized_bitmap);
    /// ```
    #[inline]
    #[must_use]
    #[cfg(feature = "alloc")]
    pub fn serialize<S: Serializer + crate::serialization::NoAlign>(&self) -> Vec<u8> {
        let mut dst = Vec::new();
        let res = self.serialize_into_vec::<S>(&mut dst);
        debug_assert_eq!(res.as_ptr(), dst.as_ptr());
        dst
    }

    /// Serializes a bitmap to a slice of bytes in format `S`, re-using existing capacity
    ///
    /// `dst` is not cleared, data is added after any existing data. Returns the added slice of `dst`.
    /// Because of alignment requirements, the serialized data may not start at the beginning of
    /// `dst`: the returned slice may not start at `dst.as_ptr()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::{Bitmap64, Portable};
    ///
    /// let original_bitmap_1: Bitmap64 = (1..5).collect();
    /// let original_bitmap_2: Bitmap64 = (1..10).collect();
    ///
    /// let mut data = Vec::new();
    /// for bitmap in [original_bitmap_1, original_bitmap_2] {
    ///     data.clear();
    ///     let serialized = bitmap.serialize_into_vec::<Portable>(&mut data);
    ///     // do something with serialized
    ///     # let _ = serialized;
    /// }
    /// ```
    #[inline]
    #[must_use]
    #[doc(alias = "roaring64_bitmap_portable_serialize")]
    #[cfg(feature = "alloc")]
    pub fn serialize_into_vec<'a, S: Serializer>(&self, dst: &'a mut Vec<u8>) -> &'a [u8] {
        S::serialize_into_vec(self, dst)
    }

    /// Serializes a bitmap to a slice of bytes in format `S`
    ///
    /// Returns the serialized data if the buffer was large enough, otherwise None.
    ///
    /// See [`Self::get_serialized_size_in_bytes`] to determine the required buffer size.
    /// Note also that some ([`crate::Frozen`]) formats require alignment, so the buffer size may need to
    /// be larger than the serialized size.
    ///
    /// See also [`Self::serialize_into_vec`] for a version that uses a Vec instead, or, for
    /// advanced use-cases, see [`Serializer::try_serialize_into`].
    #[inline]
    #[must_use]
    pub fn try_serialize_into<'a, S: Serializer>(&self, dst: &'a mut [u8]) -> Option<&'a mut [u8]> {
        S::try_serialize_into_aligned(self, dst)
    }

    /// Given a serialized bitmap as slice of bytes in format `S`, returns a `Bitmap64` instance.
    /// See example of [`Self::serialize`] function.
    ///
    /// On invalid input returns None.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::{Bitmap64, Portable};
    ///
    /// let original_bitmap: Bitmap64 = (1..5).collect();
    /// let mut buf = [0; 1024];
    /// let serialized_buffer: &[u8] = original_bitmap.try_serialize_into::<Portable>(&mut buf).unwrap();
    ///
    /// let deserialized_bitmap = Bitmap64::try_deserialize::<Portable>(serialized_buffer);
    /// assert_eq!(original_bitmap, deserialized_bitmap.unwrap());
    ///
    /// let invalid_buffer: Vec<u8> = vec![3];
    /// let deserialized_bitmap = Bitmap64::try_deserialize::<Portable>(&invalid_buffer);
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
    #[must_use]
    pub fn deserialize<D: Deserializer>(buffer: &[u8]) -> Self {
        Self::try_deserialize::<D>(buffer).unwrap_or_default()
    }

    /// Iterate over the values in the bitmap in sorted order
    ///
    /// If `f` returns `Break`, iteration will stop and the value will be returned,
    /// Otherwise, iteration continues. If `f` never returns break, `None` is returned after all values are visited.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// use std::ops::ControlFlow;
    ///
    /// let bitmap = Bitmap64::of(&[1, 2, 3, 14, 20, 21, 100]);
    /// let mut even_nums_under_50 = vec![];
    ///
    /// let first_over_50 = bitmap.for_each(|value| {
    ///     if value > 50 {
    ///        return ControlFlow::Break(value);
    ///     }
    ///     if value % 2 == 0 {
    ///         even_nums_under_50.push(value);
    ///     }
    ///     ControlFlow::Continue(())
    /// });
    ///
    /// assert_eq!(even_nums_under_50, vec![2, 14, 20]);
    /// assert_eq!(first_over_50, ControlFlow::Break(100));
    /// ```
    #[inline]
    pub fn for_each<F, O>(&self, f: F) -> core::ops::ControlFlow<O>
    where
        F: FnMut(u64) -> core::ops::ControlFlow<O>,
    {
        #[cfg(feature = "std")]
        {
            let mut callback_wrapper = crate::callback::CallbackWrapper::new(f);
            let (callback, context) = callback_wrapper.callback_and_ctx();
            unsafe {
                ffi::roaring64_bitmap_iterate(self.raw.as_ptr(), Some(callback), context);
            }
            match callback_wrapper.result() {
                Ok(cf) => cf,
                Err(e) => std::panic::resume_unwind(e),
            }
        }
        #[cfg(not(feature = "std"))]
        {
            self.iter().try_for_each(f)
        }
    }

    /// Returns statistics about the composition of a roaring bitmap64.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// let mut bitmap: Bitmap64 = (1..100).collect();
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
    /// assert_eq!(statistics.cardinality, 99);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_statistics")]
    #[must_use]
    pub fn statistics(&self) -> Statistics {
        let mut stats = MaybeUninit::<ffi::roaring64_statistics_t>::zeroed();
        unsafe {
            ffi::roaring64_bitmap_statistics(self.raw.as_ptr(), stats.as_mut_ptr());
            stats.assume_init()
        }
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
    /// use croaring::Bitmap64;
    ///
    /// let bitmap = Bitmap64::from_range(0..100);
    /// bitmap.internal_validate().unwrap();
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_internal_validate")]
    #[doc(hidden)]
    pub fn internal_validate(&self) -> Result<(), &'static str> {
        let mut error_str = ptr::null();
        let valid =
            unsafe { ffi::roaring64_bitmap_internal_validate(self.raw.as_ptr(), &mut error_str) };
        if valid {
            Ok(())
        } else {
            if error_str.is_null() {
                return Err("Unknown error");
            }
            let reason = unsafe { core::ffi::CStr::from_ptr(error_str) };
            Err(reason.to_str().unwrap_or("Invalid UTF-8 in error message"))
        }
    }
}

/// Binary Operations
impl Bitmap64 {
    /// Return true if self and other contain _any_ common elements
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let bitmap1 = Bitmap64::of(&[1, 2, 3]);
    /// let bitmap2 = Bitmap64::of(&[2, 3, 4]);
    /// assert!(bitmap1.intersect(&bitmap2));
    /// ```
    #[inline]
    #[must_use]
    #[doc(alias = "roaring64_bitmap_intersect")]
    pub fn intersect(&self, other: &Self) -> bool {
        unsafe { ffi::roaring64_bitmap_intersect(self.raw.as_ptr(), other.raw.as_ptr()) }
    }

    /// Check if a bitmap has any values set in `range`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// let bitmap = Bitmap64::of(&[1, 100, 101, u64::MAX]);
    ///
    /// assert!(bitmap.intersect_with_range(0..10));
    /// assert!(!bitmap.intersect_with_range(2..100));
    /// assert!(bitmap.intersect_with_range(999..=u64::MAX));
    ///
    /// // Empty ranges
    /// assert!(!bitmap.intersect_with_range(100..100));
    /// assert!(!bitmap.intersect_with_range(100..0));
    /// ```
    #[inline]
    #[must_use]
    #[doc(alias = "roaring64_bitmap_intersect_with_range")]
    pub fn intersect_with_range<R: RangeBounds<u64>>(&self, range: R) -> bool {
        let Some(exclusive_range) = range_to_exclusive(range) else {
            return false;
        };
        self._intersect_with_range(exclusive_range)
    }

    #[inline]
    fn _intersect_with_range(&self, exclusive_range: ExclusiveRangeRes) -> bool {
        let ExclusiveRangeRes {
            start,
            end,
            needs_max,
        } = exclusive_range;
        if needs_max && self.contains(u64::MAX) {
            return true;
        }
        unsafe { ffi::roaring64_bitmap_intersect_with_range(self.raw.as_ptr(), start, end) }
    }

    /// Computes the Jaccard index between two bitmaps
    ///
    /// This is also known as the Tanimoto distance, or the Jaccard similarity coefficient
    ///
    /// The Jaccard index is NaN if both bitmaps are empty
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let bitmap1 = Bitmap64::of(&[1, 2, 3]);
    /// let bitmap2 = Bitmap64::of(&[2, 3, 4]);
    /// assert_eq!(bitmap1.jaccard_index(&bitmap2), 0.5);
    ///
    /// let empty_bitmap = Bitmap64::new();
    /// assert!(empty_bitmap.jaccard_index(&empty_bitmap).is_nan());
    /// ```
    #[inline]
    #[must_use]
    #[doc(alias = "roaring64_bitmap_jaccard_index")]
    pub fn jaccard_index(&self, other: &Self) -> f64 {
        unsafe { ffi::roaring64_bitmap_jaccard_index(self.raw.as_ptr(), other.raw.as_ptr()) }
    }

    /// Computes the intersection between two bitmaps and returns the result
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let bitmap1 = Bitmap64::of(&[1, 2, 3]);
    /// let bitmap2 = Bitmap64::of(&[2, 3, 4]);
    /// let bitmap3 = bitmap1.and(&bitmap2);
    /// assert!(bitmap3.contains(2));
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_and")]
    #[must_use]
    pub fn and(&self, other: &Self) -> Self {
        unsafe {
            Self::take_heap(ffi::roaring64_bitmap_and(
                self.raw.as_ptr(),
                other.raw.as_ptr(),
            ))
        }
    }

    /// Computes the size of the intersection between two bitmaps
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let bitmap1 = Bitmap64::of(&[1, 2, 3]);
    /// let bitmap2 = Bitmap64::of(&[2, 3, 4]);
    /// assert_eq!(bitmap1.and_cardinality(&bitmap2), 2);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_and_cardinality")]
    #[must_use]
    pub fn and_cardinality(&self, other: &Self) -> u64 {
        unsafe { ffi::roaring64_bitmap_and_cardinality(self.raw.as_ptr(), other.raw.as_ptr()) }
    }

    /// Computes the intersection between two bitmaps and stores the result in the current bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap1 = Bitmap64::of(&[1, 2, 3]);
    /// let bitmap2 = Bitmap64::of(&[2, 3, 4]);
    /// bitmap1.and_inplace(&bitmap2);
    /// assert!(bitmap1.contains(2));
    /// assert!(bitmap1.contains(3));
    /// assert!(!bitmap1.contains(1));
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_and_inplace")]
    pub fn and_inplace(&mut self, other: &Self) {
        unsafe { ffi::roaring64_bitmap_and_inplace(self.raw.as_ptr(), other.raw.as_ptr()) }
    }

    /// Computes the union between two bitmaps and returns the result
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let bitmap1 = Bitmap64::of(&[1, 2, 3]);
    /// let bitmap2 = Bitmap64::of(&[2, 3, 4]);
    /// let bitmap3 = bitmap1.or(&bitmap2);
    /// assert_eq!(bitmap3.iter().collect::<Vec<_>>(), vec![1, 2, 3, 4]);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_or")]
    #[must_use]
    pub fn or(&self, other: &Self) -> Self {
        unsafe {
            Self::take_heap(ffi::roaring64_bitmap_or(
                self.raw.as_ptr(),
                other.raw.as_ptr(),
            ))
        }
    }

    /// Computes the size of the union between two bitmaps
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let bitmap1 = Bitmap64::of(&[1, 2, 3]);
    /// let bitmap2 = Bitmap64::of(&[2, 3, 4]);
    /// assert_eq!(bitmap1.or_cardinality(&bitmap2), 4);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_or_cardinality")]
    #[must_use]
    pub fn or_cardinality(&self, other: &Self) -> u64 {
        unsafe { ffi::roaring64_bitmap_or_cardinality(self.raw.as_ptr(), other.raw.as_ptr()) }
    }

    /// Computes the union between two bitmaps and stores the result in the current bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap1 = Bitmap64::of(&[1, 2, 3]);
    /// let bitmap2 = Bitmap64::of(&[2, 3, 4]);
    /// bitmap1.or_inplace(&bitmap2);
    /// assert_eq!(bitmap1.iter().collect::<Vec<_>>(), vec![1, 2, 3, 4]);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_or_inplace")]
    pub fn or_inplace(&mut self, other: &Self) {
        unsafe { ffi::roaring64_bitmap_or_inplace(self.raw.as_ptr(), other.raw.as_ptr()) }
    }

    /// Computes the symmetric difference (xor) between two bitmaps and returns the result
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let bitmap1 = Bitmap64::of(&[1, 2, 3]);
    /// let bitmap2 = Bitmap64::of(&[2, 3, 4]);
    /// let bitmap3 = bitmap1.xor(&bitmap2);
    /// assert_eq!(bitmap3.iter().collect::<Vec<_>>(), vec![1, 4]);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_xor")]
    #[must_use]
    pub fn xor(&self, other: &Self) -> Self {
        unsafe {
            Self::take_heap(ffi::roaring64_bitmap_xor(
                self.raw.as_ptr(),
                other.raw.as_ptr(),
            ))
        }
    }

    /// Computes the size of the symmetric difference (xor) between two bitmaps
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let bitmap1 = Bitmap64::of(&[1, 2, 3]);
    /// let bitmap2 = Bitmap64::of(&[2, 3, 4]);
    /// assert_eq!(bitmap1.xor_cardinality(&bitmap2), 2);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_xor_cardinality")]
    #[must_use]
    pub fn xor_cardinality(&self, other: &Self) -> u64 {
        unsafe { ffi::roaring64_bitmap_xor_cardinality(self.raw.as_ptr(), other.raw.as_ptr()) }
    }

    /// Computes the symmetric difference (xor) between two bitmaps and stores the result in the current bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap1 = Bitmap64::of(&[1, 2, 3]);
    /// let bitmap2 = Bitmap64::of(&[2, 3, 4]);
    /// bitmap1.xor_inplace(&bitmap2);
    /// assert_eq!(bitmap1.iter().collect::<Vec<_>>(), vec![1, 4]);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_xor_inplace")]
    pub fn xor_inplace(&mut self, other: &Self) {
        unsafe { ffi::roaring64_bitmap_xor_inplace(self.raw.as_ptr(), other.raw.as_ptr()) }
    }

    /// Computes the difference between two bitmaps and returns the result
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let bitmap1 = Bitmap64::of(&[1, 2, 3]);
    /// let bitmap2 = Bitmap64::of(&[2, 3, 4]);
    /// let bitmap3 = bitmap1.andnot(&bitmap2);
    /// assert_eq!(bitmap3.iter().collect::<Vec<_>>(), vec![1]);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_andnot")]
    #[must_use]
    pub fn andnot(&self, other: &Self) -> Self {
        unsafe {
            Self::take_heap(ffi::roaring64_bitmap_andnot(
                self.raw.as_ptr(),
                other.raw.as_ptr(),
            ))
        }
    }

    /// Computes the size of the difference between two bitmaps
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let bitmap1 = Bitmap64::of(&[1, 2, 3]);
    /// let bitmap2 = Bitmap64::of(&[2, 3, 4]);
    /// assert_eq!(bitmap1.andnot_cardinality(&bitmap2), 1);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_andnot_cardinality")]
    #[must_use]
    pub fn andnot_cardinality(&self, other: &Self) -> u64 {
        unsafe { ffi::roaring64_bitmap_andnot_cardinality(self.raw.as_ptr(), other.raw.as_ptr()) }
    }

    /// Computes the difference between two bitmaps and stores the result in the current bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap1 = Bitmap64::of(&[1, 2, 3]);
    /// let bitmap2 = Bitmap64::of(&[2, 3, 4]);
    /// bitmap1.andnot_inplace(&bitmap2);
    /// assert_eq!(bitmap1.iter().collect::<Vec<_>>(), vec![1]);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_bitmap_andnot_inplace")]
    pub fn andnot_inplace(&mut self, other: &Self) {
        unsafe { ffi::roaring64_bitmap_andnot_inplace(self.raw.as_ptr(), other.raw.as_ptr()) }
    }
}

/// Returns start, end, and whether the range also includes u64::MAX
struct ExclusiveRangeRes {
    start: u64,
    end: u64,
    needs_max: bool,
}

fn range_to_exclusive<R: RangeBounds<u64>>(range: R) -> Option<ExclusiveRangeRes> {
    let (start, inclusive_end) = range_to_inclusive(range);

    if inclusive_end < start {
        return None;
    }

    let (end, needs_max) = match inclusive_end.checked_add(1) {
        Some(i) => (i, false),
        None => (u64::MAX, true),
    };
    Some(ExclusiveRangeRes {
        start,
        end,
        needs_max,
    })
}

fn range_to_inclusive<R: RangeBounds<u64>>(range: R) -> (u64, u64) {
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
        Bound::Unbounded => u64::MAX,
    };
    (start, end)
}
