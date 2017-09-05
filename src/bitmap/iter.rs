use std::iter::{FromIterator, IntoIterator};
use std::marker::PhantomData;

use super::{Bitmap, ffi};

pub struct BitmapIterator<'a> {
    iterator: *mut ffi::roaring_uint32_iterator_s,
    phantom: PhantomData<&'a ()>,
}

impl<'a> BitmapIterator<'a> {
    fn new(bitmap: &Bitmap) -> Self {
        BitmapIterator {
            iterator: unsafe { ffi::roaring_create_iterator(bitmap.bitmap) },
            phantom: PhantomData,
        }
    }

    #[inline]
    fn current_value(&self) -> Option<u32> {
        unsafe {
            if self.has_value() {
                Some((*self.iterator).current_value)
            } else {
                None
            }
        }
    }

    #[inline]
    fn has_value(&self) -> bool {
        unsafe { (*self.iterator).has_value }
    }

    #[inline]
    fn advance(&mut self) -> bool {
        unsafe { ffi::roaring_advance_uint32_iterator(self.iterator) }
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

impl<'a> Drop for BitmapIterator<'a> {
    fn drop(&mut self) {
        unsafe { ffi::roaring_free_uint32_iterator(self.iterator) }
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
