use std::iter::{FromIterator, IntoIterator};
use std::marker::PhantomData;

use super::Bitmap;

#[derive(Clone)]
pub struct BitmapIterator<'a> {
    iterator: ffi::roaring_uint32_iterator_s,
    phantom: PhantomData<&'a Bitmap>,
}

unsafe impl Send for BitmapIterator<'_> {}
unsafe impl Sync for BitmapIterator<'_> {}

impl<'a> BitmapIterator<'a> {
    fn new(bitmap: &'a Bitmap) -> Self {
        let mut iterator = std::mem::MaybeUninit::uninit();
        unsafe {
            ffi::roaring_init_iterator(bitmap.bitmap, iterator.as_mut_ptr());
        }
        BitmapIterator {
            iterator: unsafe { iterator.assume_init() },
            phantom: PhantomData,
        }
    }

    #[inline]
    fn current_value(&self) -> Option<u32> {
        if self.has_value() {
            Some(self.iterator.current_value)
        } else {
            None
        }
    }

    #[inline]
    fn has_value(&self) -> bool {
        self.iterator.has_value
    }

    #[inline]
    fn advance(&mut self) -> bool {
        unsafe { ffi::roaring_advance_uint32_iterator(&mut self.iterator) }
    }

    /// Attempt to read many values from the iterator into `dst`
    ///
    /// Returns the number of items read from the iterator, may be `< dst.len()` iff
    /// the iterator is exhausted.
    ///
    /// This can be much more efficient than repeated iteration.
    #[inline]
    pub fn next_many(&mut self, dst: &mut [u32]) -> usize {
        let count: u32 = u32::try_from(dst.len()).unwrap_or(u32::MAX);
        let result = unsafe { ffi::roaring_read_uint32_iterator(&mut self.iterator, dst.as_mut_ptr(), count)};
        result as usize
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
    /// let mut bitmap = Bitmap::create();
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
    pub fn iter(&self) -> BitmapIterator {
        BitmapIterator::new(self)
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
        Bitmap::of(&Vec::from_iter(iter))
    }
}
