use std::iter::{FromIterator, IntoIterator};
use std::marker::PhantomData;

use super::Bitmap;

pub struct BitmapIterator<'a> {
    iterator: ffi::roaring_uint32_iterator_s,
    phantom: PhantomData<&'a Bitmap>,
}

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

impl Extend<u32> for Bitmap {
    fn extend<T: IntoIterator<Item=u32>>(&mut self, iter: T) {
        for item in iter {
            self.add(item);
        }
    }
}
