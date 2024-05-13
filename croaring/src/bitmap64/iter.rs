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

/// A cursor over a bitmap64
///
/// A Cursor is like an iterator, except that it can freely seek back-and-forth.
///
/// A cursor points at a single value in the bitmap, or at a "ghost" position,
/// either one before the beginning of the bitmap, or one after the end of the bitmap.
#[derive(Debug)]
pub struct Bitmap64Cursor<'a> {
    raw: NonNull<ffi::roaring64_iterator_t>,
    has_value: bool,
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
    fn from_raw(raw: *mut ffi::roaring64_iterator_t) -> Self {
        let raw = NonNull::new(raw).expect("Failed to allocate roaring64_iterator_t");
        let has_value = unsafe { ffi::roaring64_iterator_has_value(raw.as_ptr()) };
        Self {
            raw,
            has_value,
            _bitmap: PhantomData,
        }
    }

    fn at_first(bitmap: &'a Bitmap64) -> Self {
        let raw = unsafe { ffi::roaring64_iterator_create(bitmap.raw.as_ptr()) };
        Self::from_raw(raw)
    }

    fn at_last(bitmap: &'a Bitmap64) -> Self {
        let raw = unsafe { ffi::roaring64_iterator_create_last(bitmap.raw.as_ptr()) };
        Self::from_raw(raw)
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
        self.has_value
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
        self.has_value = unsafe { ffi::roaring64_iterator_advance(self.raw.as_ptr()) };
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
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<u64> {
        self.move_next();
        self.current()
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
        self.has_value = unsafe { ffi::roaring64_iterator_previous(self.raw.as_ptr()) };
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
        self.current()
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
    pub fn reset_to_first(self, bitmap: &Bitmap64) -> Bitmap64Cursor<'_> {
        // Don't drop `self` and free the iterator
        let this = ManuallyDrop::new(self);
        unsafe { ffi::roaring64_iterator_reinit(bitmap.raw.as_ptr(), this.raw.as_ptr()) };
        Bitmap64Cursor::from_raw(this.raw.as_ptr())
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
    pub fn reset_to_last(self, bitmap: &Bitmap64) -> Bitmap64Cursor<'_> {
        // Don't drop `self` and free the iterator
        let this = ManuallyDrop::new(self);
        unsafe { ffi::roaring64_iterator_reinit_last(bitmap.raw.as_ptr(), this.raw.as_ptr()) };
        Bitmap64Cursor::from_raw(this.raw.as_ptr())
    }

    /// Attempt to read many values from the iterator into `dst`
    ///
    /// The current value _is_ included in the output.
    ///
    /// Returns the number of items read from the iterator, may be `< dst.len()` iff
    /// the iterator is exhausted or `dst.len() > u64::MAX`.
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
    /// bitmap.add(999);
    ///
    /// let mut buf = [0; 100];
    /// let mut cursor = bitmap.cursor();
    /// assert_eq!(cursor.read_many(&mut buf), 100);
    /// // Get the first 100 items, from the original range added
    /// for (i, item) in buf.iter().enumerate() {
    ///     assert_eq!(*item, i as u64);
    /// }
    /// // Calls to next_many() can be interleaved with other cursor calls
    /// assert_eq!(cursor.current(), Some(222));
    /// assert_eq!(cursor.next(), Some(555));
    /// assert_eq!(cursor.read_many(&mut buf), 2);
    /// assert_eq!(buf[0], 555);
    /// assert_eq!(buf[1], 999);
    ///
    /// assert_eq!(cursor.current(), None);
    /// assert_eq!(cursor.read_many(&mut buf), 0);
    /// ```
    ///
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// fn print_by_chunks(bitmap: &Bitmap64) {
    ///     let mut buf = [0; 1024];
    ///     let mut iter = bitmap.cursor();
    ///     loop {
    ///         let n = iter.read_many(&mut buf);
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
    pub fn read_many(&mut self, dst: &mut [u64]) -> usize {
        let count = u64::try_from(dst.len()).unwrap_or(u64::MAX);
        let result =
            unsafe { ffi::roaring64_iterator_read(self.raw.as_ptr(), dst.as_mut_ptr(), count) };
        debug_assert!(result <= count);
        self.has_value = unsafe { ffi::roaring64_iterator_has_value(self.raw.as_ptr()) };
        result as usize
    }

    /// Reset the iterator to the first value `>= val`
    ///
    /// This can move the iterator forwards or backwards.
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// let bitmap = Bitmap64::of(&[0, 1, 100, 1000, u64::MAX]);
    /// let mut cursor = bitmap.cursor();
    /// cursor.reset_at_or_after(0);
    /// assert_eq!(cursor.current(), Some(0));
    /// cursor.reset_at_or_after(0);
    /// assert_eq!(cursor.current(), Some(0));
    ///
    /// cursor.reset_at_or_after(101);
    /// assert_eq!(cursor.current(), Some(1000));
    /// assert_eq!(cursor.next(), Some(u64::MAX));
    /// assert_eq!(cursor.next(), None);
    /// cursor.reset_at_or_after(u64::MAX);
    /// assert_eq!(cursor.current(), Some(u64::MAX));
    /// assert_eq!(cursor.next(), None);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_iterator_move_equalorlarger")]
    pub fn reset_at_or_after(&mut self, val: u64) {
        self.has_value =
            unsafe { ffi::roaring64_iterator_move_equalorlarger(self.raw.as_ptr(), val) };
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
/// use croaring::bitmap64::{Bitmap64, Bitmap64Cursor};
/// let mut bitmap = Bitmap64::of(&[1, 2, 3]);
/// let mut iter = bitmap.iter();
/// assert_eq!(iter.peek(), Some(1));
/// assert_eq!(iter.next(), Some(1));
///
/// assert_eq!(iter.peek(), Some(2));
/// let mut cursor: Bitmap64Cursor = iter.into();
/// assert_eq!(cursor.current(), Some(2));
/// ```
impl<'a> From<Bitmap64Iterator<'a>> for Bitmap64Cursor<'a> {
    fn from(iter: Bitmap64Iterator<'a>) -> Self {
        iter.cursor
    }
}

/// Converts this cursor into an iterator
///
/// The next value returned by the iterator will be the current value of the cursor (if any).
///
/// # Examples
///
/// ```
/// use croaring::bitmap64::{Bitmap64, Bitmap64Iterator};
///
/// let mut bitmap = Bitmap64::of(&[1, 2, 3]);
/// let mut cursor = bitmap.cursor();
/// assert_eq!(cursor.current(), Some(1));
///
/// let mut iter = Bitmap64Iterator::from(cursor);
/// assert_eq!(iter.next(), Some(1));
/// ```
impl<'a> From<Bitmap64Cursor<'a>> for Bitmap64Iterator<'a> {
    fn from(cursor: Bitmap64Cursor<'a>) -> Self {
        Bitmap64Iterator { cursor }
    }
}

impl<'a> Clone for Bitmap64Cursor<'a> {
    fn clone(&self) -> Self {
        let raw = unsafe { ffi::roaring64_iterator_copy(self.raw.as_ptr()) };
        Self::from_raw(raw)
    }
}

/// An iterator over the values in a bitmap
#[derive(Debug, Clone)]
pub struct Bitmap64Iterator<'a> {
    cursor: Bitmap64Cursor<'a>,
}

impl<'a> Bitmap64Iterator<'a> {
    fn new(bitmap: &'a Bitmap64) -> Self {
        Self {
            cursor: bitmap.cursor(),
        }
    }

    #[inline]
    fn advance(&mut self) {
        self.cursor.move_next();
    }

    /// Attempt to read many values from the iterator into `dst`
    ///
    /// Returns the number of items read from the iterator, may be `< dst.len()` iff
    /// the iterator is exhausted or `dst.len() > u64::MAX`.
    ///
    /// This can be much more efficient than repeated iteration.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// let mut bitmap: Bitmap64 = Bitmap64::new();
    /// bitmap.add_range(0..100);
    /// bitmap.add(222);
    /// bitmap.add(555);
    ///
    /// let mut buf = [0; 100];
    /// let mut iter = bitmap.iter();
    /// assert_eq!(iter.next_many(&mut buf), 100);
    /// // Get the first 100 items, from the original range added
    /// for (i, item) in buf.iter().enumerate() {
    ///     assert_eq!(*item, i as u64);
    /// }
    /// // Calls to next_many() can be interleaved with calls to next()
    /// assert_eq!(iter.next(), Some(222));
    /// assert_eq!(iter.next_many(&mut buf), 1);
    /// assert_eq!(buf[0], 555);
    ///
    /// assert_eq!(iter.next(), None);
    /// assert_eq!(iter.next_many(&mut buf), 0);
    /// ```
    ///
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// fn print_by_chunks(bitmap: &Bitmap64) {
    ///     let mut buf = [0; 1024];
    ///     let mut iter = bitmap.iter();
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
        self.cursor.read_many(dst)
    }

    /// Reset the iterator to the first value `>= val`
    ///
    /// This can move the iterator forwards or backwards.
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitmap64;
    ///
    /// let mut bitmap = Bitmap64::of(&[0, 1, 100, 1000, u64::MAX]);
    /// let mut iter = bitmap.iter();
    /// iter.reset_at_or_after(0);
    /// assert_eq!(iter.next(), Some(0));
    /// iter.reset_at_or_after(0);
    /// assert_eq!(iter.next(), Some(0));
    ///
    /// iter.reset_at_or_after(101);
    /// assert_eq!(iter.next(), Some(1000));
    /// assert_eq!(iter.next(), Some(u64::MAX));
    /// assert_eq!(iter.next(), None);
    /// iter.reset_at_or_after(u64::MAX);
    /// assert_eq!(iter.next(), Some(u64::MAX));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    #[doc(alias = "roaring64_iterator_move_equalorlarger")]
    pub fn reset_at_or_after(&mut self, val: u64) {
        self.cursor.reset_at_or_after(val);
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
        self.cursor.current()
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
