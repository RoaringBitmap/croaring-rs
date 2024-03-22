use std::marker::PhantomData;
use std::mem::MaybeUninit;

use super::Bitmap;

/// A cusrsr over the values of a bitmap
///
/// A Cursor is like an iterator, except that it can freely seek back-and-forth.
///
/// A cursor points at a single value in the bitmap, or at a "ghost" position,
/// either one before the beginning of the bitmap, or one after the end of the bitmap.
#[derive(Debug, Clone)]
pub struct BitmapCursor<'a> {
    raw: ffi::roaring_uint32_iterator_t,
    _bitmap: PhantomData<&'a Bitmap>,
}

unsafe impl Send for BitmapCursor<'_> {}

unsafe impl Sync for BitmapCursor<'_> {}

impl<'a> BitmapCursor<'a> {
    #[inline]
    fn from_raw(raw: ffi::roaring_uint32_iterator_t) -> Self {
        BitmapCursor {
            raw,
            _bitmap: PhantomData,
        }
    }

    fn at_first(bitmap: &'a Bitmap) -> Self {
        let mut raw = MaybeUninit::<ffi::roaring_uint32_iterator_s>::uninit();
        unsafe { ffi::roaring_iterator_init(&bitmap.bitmap, raw.as_mut_ptr()) };
        Self::from_raw(unsafe { raw.assume_init() })
    }

    fn at_last(bitmap: &'a Bitmap) -> Self {
        let mut raw = MaybeUninit::<ffi::roaring_uint32_iterator_s>::uninit();
        unsafe { ffi::roaring_iterator_init_last(&bitmap.bitmap, raw.as_mut_ptr()) };
        Self::from_raw(unsafe { raw.assume_init() })
    }

    /// Returns true if the cursor is pointing at a value in the bitmap.
    ///
    /// If this returns false, then the cursor is pointing at a "ghost" position,
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    /// let mut bitmap = Bitmap::new();
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
        self.raw.has_value
    }

    /// Returns the value at the cursor, if any.
    ///
    /// If the cursor is not pointing at a value, then this returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    /// let mut bitmap = Bitmap::new();
    /// bitmap.add(1);
    /// let mut cursor = bitmap.cursor();
    /// assert_eq!(cursor.current(), Some(1));
    /// cursor.move_next();
    /// assert_eq!(cursor.current(), None);
    /// ```
    #[inline]
    pub fn current(&self) -> Option<u32> {
        if self.has_value() {
            Some(self.raw.current_value)
        } else {
            None
        }
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
    /// use croaring::Bitmap;
    /// let mut bitmap = Bitmap::of(&[1, 2, 3]);
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
        unsafe { ffi::roaring_uint32_iterator_advance(&mut self.raw) };
    }

    /// Moves the cursor to the next value in the bitmap, and returns the value (if any)
    ///
    /// This is equivalent to calling [`Self::move_next`] followed by [`Self::current`].
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    /// let mut bitmap = Bitmap::of(&[1, 2, 3]);
    /// let mut cursor = bitmap.cursor();
    /// assert_eq!(cursor.current(), Some(1));
    /// assert_eq!(cursor.next(), Some(2));
    /// assert_eq!(cursor.next(), Some(3));
    /// assert_eq!(cursor.next(), None);
    /// ```
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<u32> {
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
    /// use croaring::Bitmap;
    /// let mut bitmap = Bitmap::of(&[1, 2, 3]);
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
        unsafe { ffi::roaring_uint32_iterator_previous(&mut self.raw) };
    }

    /// Moves the cursor to the previous value in the bitmap, and returns the value (if any)
    ///
    /// This is equivalent to calling [`Self::move_prev`] followed by [`Self::current`].
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    /// let mut bitmap = Bitmap::of(&[1, 2, 3]);
    /// let mut cursor = bitmap.cursor_to_last();
    /// assert_eq!(cursor.current(), Some(3));
    /// assert_eq!(cursor.prev(), Some(2));
    /// assert_eq!(cursor.prev(), Some(1));
    /// assert_eq!(cursor.prev(), None);
    /// ```
    #[inline]
    pub fn prev(&mut self) -> Option<u32> {
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
    /// use croaring::Bitmap;
    /// let mut bitmap1 = Bitmap::of(&[1, 2, 3]);
    /// let bitmap2 = Bitmap::of(&[4, 5, 6]);
    /// let cursor = bitmap1.cursor();
    /// assert_eq!(cursor.current(), Some(1));
    /// let cursor = cursor.reset_to_first(&bitmap2);
    /// assert_eq!(cursor.current(), Some(4));
    /// // Cursor is no longer borrowing from bitmap1
    /// bitmap1.add(100);
    /// ```
    #[must_use]
    pub fn reset_to_first(self, bitmap: &Bitmap) -> BitmapCursor<'_> {
        BitmapCursor::at_first(bitmap)
    }

    /// Resets this cursor to the last value in the bitmap.
    ///
    /// The bitmap does not have to be the same bitmap that this cursor was created from:
    /// this allows you to reuse a cursor for multiple bitmaps
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    /// let mut bitmap1 = Bitmap::of(&[1, 2, 3]);
    /// let bitmap2 = Bitmap::of(&[4, 5, 6]);
    /// let cursor = bitmap1.cursor_to_last();
    /// assert_eq!(cursor.current(), Some(3));
    /// let cursor = cursor.reset_to_last(&bitmap2);
    /// assert_eq!(cursor.current(), Some(6));
    /// ```
    #[must_use]
    pub fn reset_to_last(self, bitmap: &Bitmap) -> BitmapCursor<'_> {
        BitmapCursor::at_last(bitmap)
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
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::new();
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
    ///     assert_eq!(*item, i as u32);
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
    /// use croaring::Bitmap;
    ///
    /// fn print_by_chunks(bitmap: &Bitmap) {
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
    /// # print_by_chunks(&Bitmap::of(&[1, 2, 8, 20, 1000]));
    /// ```
    #[inline]
    #[doc(alias = "roaring_uint32_iterator_read")]
    pub fn read_many(&mut self, dst: &mut [u32]) -> usize {
        let count = u32::try_from(dst.len()).unwrap_or(u32::MAX);
        let result =
            unsafe { ffi::roaring_uint32_iterator_read(&mut self.raw, dst.as_mut_ptr(), count) };
        debug_assert!(result <= count);
        result as usize
    }

    /// Reset the iterator to the first value `>= val`
    ///
    /// This can move the iterator forwards or backwards.
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap = Bitmap::of(&[0, 1, 100, 1000, u32::MAX]);
    /// let mut cursor = bitmap.cursor();
    /// cursor.reset_at_or_after(0);
    /// assert_eq!(cursor.current(), Some(0));
    /// cursor.reset_at_or_after(0);
    /// assert_eq!(cursor.current(), Some(0));
    ///
    /// cursor.reset_at_or_after(101);
    /// assert_eq!(cursor.current(), Some(1000));
    /// assert_eq!(cursor.next(), Some(u32::MAX));
    /// assert_eq!(cursor.next(), None);
    /// cursor.reset_at_or_after(u32::MAX);
    /// assert_eq!(cursor.current(), Some(u32::MAX));
    /// assert_eq!(cursor.next(), None);
    /// ```
    #[inline]
    #[doc(alias = "roaring_uint32_iterator_move_equalorlarger")]
    pub fn reset_at_or_after(&mut self, val: u32) {
        unsafe { ffi::roaring_uint32_iterator_move_equalorlarger(&mut self.raw, val) };
    }
}

/// Iterator over the values of a bitmap
#[derive(Clone)]
pub struct BitmapIterator<'a> {
    cursor: BitmapCursor<'a>,
}

impl<'a> BitmapIterator<'a> {
    fn new(bitmap: &'a Bitmap) -> Self {
        Self {
            cursor: BitmapCursor::at_first(bitmap),
        }
    }

    #[inline]
    fn current_value(&self) -> Option<u32> {
        self.cursor.current()
    }

    #[inline]
    fn advance(&mut self) {
        self.cursor.move_next();
    }

    /// Attempt to read many values from the iterator into `dst`
    ///
    /// Returns the number of items read from the iterator, may be `< dst.len()` iff
    /// the iterator is exhausted or `dst.len() > u32::MAX`.
    ///
    /// This can be much more efficient than repeated iteration.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap: Bitmap = Bitmap::new();
    /// bitmap.add_range(0..100);
    /// bitmap.add(222);
    /// bitmap.add(555);
    ///
    /// let mut buf = [0; 100];
    /// let mut iter = bitmap.iter();
    /// assert_eq!(iter.next_many(&mut buf), 100);
    /// // Get the first 100 items, from the original range added
    /// for (i, item) in buf.iter().enumerate() {
    ///     assert_eq!(*item, i as u32);
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
    /// use croaring::Bitmap;
    ///
    /// fn print_by_chunks(bitmap: &Bitmap) {
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
    /// # print_by_chunks(&Bitmap::of(&[1, 2, 8, 20, 1000]));
    /// ```
    #[inline]
    #[doc(alias = "roaring_uint32_iterator_read")]
    pub fn next_many(&mut self, dst: &mut [u32]) -> usize {
        self.cursor.read_many(dst)
    }

    /// Reset the iterator to the first value `>= val`
    ///
    /// This can move the iterator forwards or backwards.
    ///
    /// # Examples
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::of(&[0, 1, 100, 1000, u32::MAX]);
    /// let mut iter = bitmap.iter();
    /// iter.reset_at_or_after(0);
    /// assert_eq!(iter.next(), Some(0));
    /// iter.reset_at_or_after(0);
    /// assert_eq!(iter.next(), Some(0));
    ///
    /// iter.reset_at_or_after(101);
    /// assert_eq!(iter.next(), Some(1000));
    /// assert_eq!(iter.next(), Some(u32::MAX));
    /// assert_eq!(iter.next(), None);
    /// iter.reset_at_or_after(u32::MAX);
    /// assert_eq!(iter.next(), Some(u32::MAX));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    #[doc(alias = "roaring_uint32_iterator_move_equalorlarger")]
    pub fn reset_at_or_after(&mut self, val: u32) {
        self.cursor.reset_at_or_after(val);
    }

    /// Peek at the next value to be returned by the iterator (if any), without consuming it
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    /// let mut bitmap = Bitmap::of(&[1, 2, 3]);
    /// let mut iter = bitmap.iter();
    /// assert_eq!(iter.peek(), Some(1));
    /// assert_eq!(iter.next(), Some(1));
    /// ```
    #[inline]
    pub fn peek(&self) -> Option<u32> {
        self.cursor.current()
    }
}

impl<'a> Iterator for BitmapIterator<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_value() {
            Some(value) => {
                self.advance();

                Some(value)
            }
            None => None,
        }
    }
}

impl Bitmap {
    /// Returns an iterator over each value stored in the bitmap.
    /// Returned values are ordered in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::new();
    /// bitmap.add(4);
    /// bitmap.add(3);
    /// bitmap.add(2);
    /// let mut iterator = bitmap.iter();
    ///
    /// assert_eq!(iterator.next(), Some(2));
    /// assert_eq!(iterator.next(), Some(3));
    /// assert_eq!(iterator.next(), Some(4));
    /// assert_eq!(iterator.next(), None);
    /// ```
    #[inline]
    #[doc(alias = "roaring_init_iterator")]
    #[must_use]
    pub fn iter(&self) -> BitmapIterator {
        BitmapIterator::new(self)
    }

    /// Returns a cursor pointing at the first value in the bitmap.
    ///
    /// See [`BitmapCursor`] for more details.
    #[inline]
    #[must_use]
    pub fn cursor(&self) -> BitmapCursor {
        BitmapCursor::at_first(self)
    }

    /// Returns a cursor pointing at the last value in the bitmap.
    ///
    /// See [`BitmapCursor`] for more details.
    #[inline]
    #[must_use]
    pub fn cursor_to_last(&self) -> BitmapCursor {
        BitmapCursor::at_last(self)
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
/// use croaring::bitmap::{Bitmap, BitmapCursor};
/// let mut bitmap = Bitmap::of(&[1, 2, 3]);
/// let mut iter = bitmap.iter();
/// assert_eq!(iter.peek(), Some(1));
/// assert_eq!(iter.next(), Some(1));
///
/// assert_eq!(iter.peek(), Some(2));
/// let mut cursor: BitmapCursor = iter.into();
/// assert_eq!(cursor.current(), Some(2));
/// ```
impl<'a> From<BitmapIterator<'a>> for BitmapCursor<'a> {
    fn from(iterator: BitmapIterator<'a>) -> Self {
        iterator.cursor
    }
}

/// Converts this cursor into an iterator
///
/// The next value returned by the iterator will be the current value of the cursor (if any).
///
/// # Examples
///
/// ```
/// use croaring::bitmap::{Bitmap, BitmapIterator};
///
/// let mut bitmap = Bitmap::of(&[1, 2, 3]);
/// let mut cursor = bitmap.cursor();
/// assert_eq!(cursor.current(), Some(1));
///
/// let mut iter = BitmapIterator::from(cursor);
/// assert_eq!(iter.next(), Some(1));
/// ```
impl<'a> From<BitmapCursor<'a>> for BitmapIterator<'a> {
    fn from(cursor: BitmapCursor<'a>) -> Self {
        BitmapIterator { cursor }
    }
}

impl FromIterator<u32> for Bitmap {
    /// Convenience method for creating bitmap from iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap: Bitmap = (1..3).collect();
    ///
    /// assert!(!bitmap.is_empty());
    /// assert!(bitmap.contains(1));
    /// assert!(bitmap.contains(2));
    /// assert_eq!(bitmap.cardinality(), 2);
    /// ```
    fn from_iter<I: IntoIterator<Item = u32>>(iter: I) -> Self {
        let mut bitmap = Bitmap::new();
        bitmap.extend(iter);
        bitmap
    }
}

impl Extend<u32> for Bitmap {
    fn extend<T: IntoIterator<Item = u32>>(&mut self, iter: T) {
        let mut ctx = MaybeUninit::<ffi::roaring_bulk_context_t>::zeroed();
        iter.into_iter().for_each(|item| unsafe {
            ffi::roaring_bitmap_add_bulk(&mut self.bitmap, ctx.as_mut_ptr(), item);
        });
    }
}
