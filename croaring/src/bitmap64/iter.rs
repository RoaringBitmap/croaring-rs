use crate::Bitmap64;
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ptr::NonNull;

impl FromIterator<u64> for Bitmap64 {
    /// Convenience method for creating bitmap from iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// let bitmap: Bitmap64 = (1..3).collect();
    ///
    /// assert!(!bitmap.is_empty());
    /// assert!(bitmap.contains(1));
    /// assert!(bitmap.contains(2));
    /// assert_eq!(bitmap.cardinality(), 2);
    /// ```
    fn from_iter<I: IntoIterator<Item = u64>>(iter: I) -> Self {
        let mut bitmap = Bitmap64::new();
        bitmap.extend(iter);
        bitmap
    }
}
impl Extend<u64> for Bitmap64 {
    #[doc(alias = "roaring64_bitmap_add_bulk")]
    fn extend<T: IntoIterator<Item = u64>>(&mut self, iter: T) {
        let mut ctx = MaybeUninit::<ffi::roaring64_bulk_context_t>::zeroed();
        iter.into_iter().for_each(|value| unsafe {
            ffi::roaring64_bitmap_add_bulk(self.raw.as_ptr(), ctx.as_mut_ptr(), value);
        });
    }
}

// TODO: is this needed: https://github.com/RoaringBitmap/CRoaring/pull/558#discussion_r1464188393
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Location {
    BeforeStart,
    AfterEnd,
    Inside,
}

/// A cursor over a bitmap64
///
/// A Cursor is like an iterator, except that it can freely seek back-and-forth.
///
/// A cursor points at a single value in the bitmap, or at a "ghost" position,
/// either one before the beginning of the bitmap, or one after the end of the bitmap.
pub struct Bitmap64Cursor<'a> {
    raw: NonNull<ffi::roaring64_iterator_t>,
    loc: Location,
    _bitmap: PhantomData<&'a Bitmap64>,
}

unsafe impl Send for Bitmap64Cursor<'_> {}
unsafe impl Sync for Bitmap64Cursor<'_> {}

impl Drop for Bitmap64Cursor<'_> {
    fn drop(&mut self) {
        unsafe {
            ffi::roaring64_iterator_free(self.raw.as_ptr());
        }
    }
}

impl<'a> Bitmap64Cursor<'a> {
    fn at_first(bitmap: &'a Bitmap64) -> Self {
        let raw = unsafe { ffi::roaring64_iterator_create(bitmap.raw.as_ptr()) };
        let raw = NonNull::new(raw).expect("Failed to allocate roaring64_iterator_t");
        Self {
            raw,
            loc: Location::Inside,
            _bitmap: PhantomData,
        }
    }

    fn at_last(bitmap: &'a Bitmap64) -> Self {
        let raw = unsafe { ffi::roaring64_iterator_create_last(bitmap.raw.as_ptr()) };
        let raw = NonNull::new(raw).expect("Failed to allocate roaring64_iterator_t");
        Self {
            raw,
            loc: Location::Inside,
            _bitmap: PhantomData,
        }
    }

    /// Returns true if the cursor is pointing at a value in the bitmap.
    ///
    /// If this returns false, then the cursor is pointing at a "ghost" position,
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap = Bitmap64::new();
    /// assert!(!bitmap.cursor().has_value());
    ///
    /// bitmap.add(1);
    /// let mut cursor = bitmap.cursor();
    /// assert!(cursor.has_value());
    /// assert_eq!(cursor.current(), Some(1));
    /// cursor.move_next();
    /// assert!(!cursor.has_value());
    /// ```
    #[inline]
    pub fn has_value(&self) -> bool {
        unsafe { ffi::roaring64_iterator_has_value(self.raw.as_ptr()) }
    }

    /// Returns the value at the cursor, if any.
    ///
    /// If the cursor is not pointing at a value, then this returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap = Bitmap64::new();
    /// bitmap.add(1);
    /// let mut cursor = bitmap.cursor();
    /// assert_eq!(cursor.current(), Some(1));
    /// cursor.move_next();
    /// assert_eq!(cursor.current(), None);
    /// ```
    #[inline]
    pub fn current(&self) -> Option<u64> {
        if self.has_value() {
            Some(unsafe { self.current_unchecked() })
        } else {
            None
        }
    }

    unsafe fn current_unchecked(&self) -> u64 {
        ffi::roaring64_iterator_value(self.raw.as_ptr())
    }

    /// Moves the cursor to the next value in the bitmap
    ///
    /// If the cursor is already past the end of the bitmap, then this does nothing.
    ///
    /// If the cursor is at the ghost position before the beginning of the bitmap,
    /// then this moves the cursor to the first value in the bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap = Bitmap64::of(&[1, 2, 3]);
    /// let mut cursor = bitmap.cursor();
    /// assert_eq!(cursor.current(), Some(1));
    /// cursor.move_prev();
    /// assert_eq!(cursor.current(), None);
    /// // TODO: This doesn't work
    /// cursor.move_next();
    /// assert_eq!(cursor.current(), Some(1));
    /// cursor.move_next();
    /// assert_eq!(cursor.current(), Some(2));
    /// cursor.move_next();
    /// assert_eq!(cursor.current(), Some(3));
    /// cursor.move_next();
    /// assert_eq!(cursor.current(), None);
    /// ```
    #[inline]
    pub fn move_next(&mut self) {
        match self.loc {
            Location::BeforeStart => self.loc = Location::Inside,
            Location::Inside => {}
            Location::AfterEnd => return,
        }

        let has_value = unsafe { ffi::roaring64_iterator_advance(self.raw.as_ptr()) };

        if !has_value {
            self.loc = Location::AfterEnd;
        }
    }

    /// Moves the cursor to the next value in the bitmap, and returns the value (if any)
    ///
    /// This is equivalent to calling [`Self::move_next`] followed by [`Self::current`].
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap = Bitmap64::of(&[1, 2, 3]);
    /// let mut cursor = bitmap.cursor();
    /// assert_eq!(cursor.current(), Some(1));
    /// assert_eq!(cursor.next(), Some(2));
    /// assert_eq!(cursor.next(), Some(3));
    /// assert_eq!(cursor.next(), None);
    /// ```
    #[inline]
    pub fn next(&mut self) -> Option<u64> {
        self.move_next();

        // We know that `move_next` will have updated the location to either `Inside` or `AfterEnd`
        // based on if the iterator has a value or not.
        if self.loc != Location::AfterEnd {
            Some(unsafe { self.current_unchecked() })
        } else {
            None
        }
    }

    /// Moves the cursor to the previous value in the bitmap
    ///
    /// If the cursor is already before the beginning of the bitmap, then this does nothing.
    ///
    /// If the cursor is at the ghost position after the end of the bitmap,
    /// then this moves the cursor to the last value in the bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap = Bitmap64::of(&[1, 2, 3]);
    /// let mut cursor = bitmap.cursor_to_last();
    /// assert_eq!(cursor.current(), Some(3));
    /// cursor.move_next();
    /// assert_eq!(cursor.current(), None);
    /// cursor.move_prev();
    /// assert_eq!(cursor.current(), Some(3));
    /// cursor.move_prev();
    /// assert_eq!(cursor.current(), Some(2));
    /// cursor.move_prev();
    /// assert_eq!(cursor.current(), Some(1));
    /// cursor.move_prev();
    /// assert_eq!(cursor.current(), None);
    /// ```
    #[inline]
    pub fn move_prev(&mut self) {
        match self.loc {
            Location::BeforeStart => return,
            Location::Inside => {}
            Location::AfterEnd => self.loc = Location::Inside,
        }
        let has_value = unsafe { ffi::roaring64_iterator_previous(self.raw.as_ptr()) };

        if !has_value {
            self.loc = Location::BeforeStart;
        }
    }

    /// Moves the cursor to the previous value in the bitmap, and returns the value (if any)
    ///
    /// This is equivalent to calling [`Self::move_prev`] followed by [`Self::current`].
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap = Bitmap64::of(&[1, 2, 3]);
    /// let mut cursor = bitmap.cursor_to_last();
    /// assert_eq!(cursor.current(), Some(3));
    /// assert_eq!(cursor.prev(), Some(2));
    /// assert_eq!(cursor.prev(), Some(1));
    /// assert_eq!(cursor.prev(), None);
    /// ```
    #[inline]
    pub fn prev(&mut self) -> Option<u64> {
        self.move_prev();

        // We know that `move_prev` will have updated the location to either `Inside` or `BeforeStart`
        // based on if the iterator has a value or not.
        if self.loc != Location::BeforeStart {
            Some(unsafe { self.current_unchecked() })
        } else {
            None
        }
    }

    /// Resets this cursor to the first value in the bitmap.
    ///
    /// The bitmap does not have to be the same bitmap that this cursor was created from:
    /// this allows you to reuse a cursor for multiple bitmaps
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap1 = Bitmap64::of(&[1, 2, 3]);
    /// let bitmap2 = Bitmap64::of(&[4, 5, 6]);
    /// let cursor = bitmap1.cursor();
    /// assert_eq!(cursor.current(), Some(1));
    /// let cursor = cursor.reset_to_first(&bitmap2);
    /// assert_eq!(cursor.current(), Some(4));
    /// // Cursor is no longer borrowing from bitmap1
    /// bitmap1.add(100);
    /// ```
    #[must_use]
    pub fn reset_to_first<'b>(self, bitmap: &'b Bitmap64) -> Bitmap64Cursor<'b> {
        // Don't drop `self` and free the iterator
        let this = ManuallyDrop::new(self);
        unsafe { ffi::roaring64_iterator_reinit(bitmap.raw.as_ptr(), this.raw.as_ptr()) };
        Bitmap64Cursor {
            raw: this.raw,
            loc: Location::Inside,
            _bitmap: PhantomData,
        }
    }

    /// Resets this cursor to the last value in the bitmap.
    ///
    /// The bitmap does not have to be the same bitmap that this cursor was created from:
    /// this allows you to reuse a cursor for multiple bitmaps
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap1 = Bitmap64::of(&[1, 2, 3]);
    /// let bitmap2 = Bitmap64::of(&[4, 5, 6]);
    /// let cursor = bitmap1.cursor_to_last();
    /// assert_eq!(cursor.current(), Some(3));
    /// let cursor = cursor.reset_to_last(&bitmap2);
    /// assert_eq!(cursor.current(), Some(6));
    /// ```
    #[must_use]
    pub fn reset_to_last<'b>(self, bitmap: &'b Bitmap64) -> Bitmap64Cursor<'b> {
        // Don't drop `self` and free the iterator
        let this = ManuallyDrop::new(self);
        unsafe { ffi::roaring64_iterator_reinit_last(bitmap.raw.as_ptr(), this.raw.as_ptr()) };
        Bitmap64Cursor {
            raw: this.raw,
            loc: Location::Inside,
            _bitmap: PhantomData,
        }
    }

    /// Attempt to read many values from the iterator into `dst`
    ///
    /// The current value _is_ included in the output.
    ///
    /// Returns the number of items read from the iterator, may be `< dst.len()` iff
    /// the iterator is exhausted or `dst.len() > u32::MAX`.
    ///
    /// This can be much more efficient than repeated iteration.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// let mut bitmap = Bitmap64::new();
    /// bitmap.add_range(0..100);
    /// bitmap.add(222);
    /// bitmap.add(555);
    ///
    /// let mut buf = [0; 100];
    /// let mut cursor = bitmap.cursor();
    /// assert_eq!(cursor.next_many(&mut buf), 100);
    /// // Get the first 100 items, from the original range added
    /// for (i, item) in buf.iter().enumerate() {
    ///     assert_eq!(*item, i as u64);
    /// }
    /// // Calls to next_many() can be interleaved with other cursor calls
    /// assert_eq!(cursor.next(), Some(222));
    /// assert_eq!(cursor.next_many(&mut buf), 1);
    /// assert_eq!(buf[0], 555);
    ///
    /// assert_eq!(cursor.next(), None);
    /// assert_eq!(cursor.next_many(&mut buf), 0);
    /// ```
    ///
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// fn print_by_chunks(bitmap: &Bitmap64) {
    ///     let mut buf = [0; 1024];
    ///     let mut iter = bitmap.cursor();
    ///     loop {
    ///         let n = iter.next_many(&mut buf);
    ///         if n == 0 {
    ///             break;
    ///         }
    ///         println!("{:?}", &buf[..n]);
    ///     }
    /// }
    ///
    /// # print_by_chunks(&Bitmap64::of(&[1, 2, 8, 20, 1000]));
    /// ```
    #[inline]
    #[doc(alias = "roaring64_iterator_read")]
    pub fn next_many(&mut self, dst: &mut [u64]) -> usize {
        let count = u64::try_from(dst.len()).unwrap_or(u64::MAX);
        let result =
            unsafe { ffi::roaring64_iterator_read(self.raw.as_ptr(), dst.as_mut_ptr(), count) };
        debug_assert!(result <= count);
        if !self.has_value() {
            self.loc = Location::AfterEnd;
        }
        result as usize
    }

    // TODO: Can this move backward like the 32 bit version? https://github.com/RoaringBitmap/CRoaring/pull/558#issuecomment-1907301009
    /// Reset the iterator to the first value `>= val`
    ///
    /// This can move the iterator forwards or backwards.
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// let mut bitmap = Bitmap64::of(&[0, 1, 100, 1000, u64::MAX]);
    /// let mut cursor = bitmap.cursor();
    /// cursor.reset_at_or_after(0);
    /// assert_eq!(cursor.next(), Some(0));
    /// cursor.reset_at_or_after(0);
    /// assert_eq!(cursor.next(), Some(0));
    ///
    /// cursor.reset_at_or_after(101);
    /// assert_eq!(cursor.next(), Some(1000));
    /// assert_eq!(cursor.next(), Some(u64::MAX));
    /// assert_eq!(cursor.next(), None);
    /// cursor.reset_at_or_after(u64::MAX);
    /// assert_eq!(cursor.next(), Some(u64::MAX));
    /// assert_eq!(cursor.next(), None);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_iterator_move_equalorlarger")]
    pub fn reset_at_or_after(&mut self, val: u64) {
        let has_value =
            unsafe { ffi::roaring64_iterator_move_equalorlarger(self.raw.as_ptr(), val) };
        if !has_value {
            self.loc = Location::AfterEnd;
        } else {
            self.loc = Location::Inside;
        }
    }
}

impl<'a> From<Bitmap64Iterator<'a>> for Bitmap64Cursor<'a> {
    fn from(iter: Bitmap64Iterator<'a>) -> Self {
        iter.into_cursor()
    }
}

impl<'a> Clone for Bitmap64Cursor<'a> {
    fn clone(&self) -> Self {
        let raw = unsafe { ffi::roaring64_iterator_copy(self.raw.as_ptr()) };
        let raw = NonNull::new(raw).expect("Failed to allocate roaring64_iterator_t");
        Self {
            raw,
            loc: self.loc,
            _bitmap: self._bitmap,
        }
    }
}

/// An iterator over the values in a bitmap
pub struct Bitmap64Iterator<'a> {
    raw: NonNull<ffi::roaring64_iterator_t>,
    has_value: bool,
    _bitmap: PhantomData<&'a Bitmap64>,
}

impl<'a> Bitmap64Iterator<'a> {
    fn new(bitmap: &'a Bitmap64) -> Self {
        let raw = unsafe { ffi::roaring64_iterator_create(bitmap.raw.as_ptr()) };
        let raw = NonNull::new(raw).expect("Failed to allocate roaring64_iterator_t");
        Self {
            raw,
            has_value: unsafe { ffi::roaring64_iterator_has_value(raw.as_ptr()) },
            _bitmap: PhantomData,
        }
    }

    #[inline]
    fn advance(&mut self) {
        self.has_value = unsafe { ffi::roaring64_iterator_advance(self.raw.as_ptr()) };
    }

    /// Peek at the next value to be returned by the iterator (if any), without consuming it
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap = Bitmap64::of(&[1, 2, 3]);
    /// let mut iter = bitmap.iter();
    /// assert_eq!(iter.peek(), Some(1));
    /// assert_eq!(iter.next(), Some(1));
    /// ```
    #[inline]
    pub fn peek(&self) -> Option<u64> {
        if self.has_value {
            Some(unsafe { ffi::roaring64_iterator_value(self.raw.as_ptr()) })
        } else {
            None
        }
    }

    /// Converts this iterator into a cursor
    ///
    /// The cursor's current value will be the the item which would have been returned by the next call to `next()`
    /// or one past the end of the bitmap if the iterator is exhausted.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    /// let mut bitmap = Bitmap64::of(&[1, 2, 3]);
    /// let mut iter = bitmap.iter();
    /// assert_eq!(iter.peek(), Some(1));
    /// assert_eq!(iter.next(), Some(1));
    ///
    /// assert_eq!(iter.peek(), Some(2));
    /// let mut cursor = iter.into_cursor();
    /// assert_eq!(cursor.current(), Some(2));
    /// ```
    pub fn into_cursor(self) -> Bitmap64Cursor<'a> {
        let this = ManuallyDrop::new(self);
        Bitmap64Cursor {
            raw: this.raw,
            loc: if this.has_value {
                Location::Inside
            } else {
                Location::AfterEnd
            },
            _bitmap: this._bitmap,
        }
    }
}

impl<'a> Iterator for Bitmap64Iterator<'a> {
    type Item = u64;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.peek() {
            Some(value) => {
                self.advance();

                Some(value)
            }
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let min_size = usize::from(self.has_value);
        (min_size, None)
    }
}

impl Bitmap64 {
    /// Returns an iterator over the values in the bitmap.
    #[inline]
    #[must_use]
    pub fn iter(&self) -> Bitmap64Iterator {
        Bitmap64Iterator::new(self)
    }

    /// Returns a cursor pointing at the first value in the bitmap.
    ///
    /// See [`Bitmap64Cursor`] for more details.
    #[inline]
    #[must_use]
    pub fn cursor(&self) -> Bitmap64Cursor {
        Bitmap64Cursor::at_first(self)
    }

    /// Returns a cursor pointing at the last value in the bitmap.
    ///
    /// See [`Bitmap64Cursor`] for more details.
    #[inline]
    #[must_use]
    pub fn cursor_to_last(&self) -> Bitmap64Cursor {
        Bitmap64Cursor::at_last(self)
    }
}

impl<'a> Clone for Bitmap64Iterator<'a> {
    fn clone(&self) -> Self {
        let raw = unsafe { ffi::roaring64_iterator_copy(self.raw.as_ptr()) };
        let raw = NonNull::new(raw).expect("Failed to allocate roaring64_iterator_t");
        Self {
            raw,
            has_value: self.has_value,
            _bitmap: self._bitmap,
        }
    }
}
