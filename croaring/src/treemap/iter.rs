use super::util;
use crate::bitmap::BitmapIterator;
use crate::{Bitmap, Treemap};
use alloc::collections::btree_map;
use core::iter;

struct To64Iter<'a> {
    key: u32,
    iterator: BitmapIterator<'a>,
}

impl<'a> Iterator for To64Iter<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        self.iterator.next().map(|n| util::join(self.key, n))
    }
}

fn to64iter<'a>((key, bitmap): (&'a u32, &'a Bitmap)) -> To64Iter<'a> {
    assert!(!bitmap.is_empty(), "empty bitmap at {key}");
    To64Iter {
        key: *key,
        iterator: bitmap.iter(),
    }
}

type InnerIter<'a> = iter::FlatMap<
    btree_map::Iter<'a, u32, Bitmap>,
    To64Iter<'a>,
    fn((&'a u32, &'a Bitmap)) -> To64Iter<'a>,
>;

/// Iterator over values stored in the treemap
///
/// Values are ordered in ascending order
pub struct TreemapIterator<'a> {
    iter: InnerIter<'a>,
}

impl<'a> TreemapIterator<'a> {
    fn new(treemap: &'a Treemap) -> Self {
        let iter = treemap.map.iter().flat_map(to64iter as _);

        TreemapIterator { iter }
    }
}

impl<'a> Iterator for TreemapIterator<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        self.iter.next()
    }
}

impl Treemap {
    /// Returns an iterator over each value stored in the bitmap.
    /// Returned values are ordered in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::u64;
    /// use croaring::Treemap;
    ///
    /// let mut treemap = Treemap::new();
    /// treemap.add(4);
    /// treemap.add(3);
    /// treemap.add(2);
    /// treemap.add(2);
    /// treemap.add(u64::MAX);
    /// let mut iterator = treemap.iter();
    ///
    /// assert_eq!(iterator.next(), Some(2));
    /// assert_eq!(iterator.next(), Some(3));
    /// assert_eq!(iterator.next(), Some(4));
    /// assert_eq!(iterator.next(), Some(u64::MAX));
    /// assert_eq!(iterator.next(), None);
    /// ```
    #[must_use]
    pub fn iter(&self) -> TreemapIterator<'_> {
        TreemapIterator::new(self)
    }
}

impl FromIterator<u64> for Treemap {
    /// Convenience method for creating treemap from an iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{u32, u64};
    /// use croaring::Treemap;
    ///
    /// let treemap: Treemap = (1..3).chain(u64::from(u32::MAX)+1..u64::from(u32::MAX)+10).collect();
    ///
    /// assert!(!treemap.is_empty());
    /// assert!(treemap.contains(1));
    /// assert!(treemap.contains(2));
    /// assert!(treemap.contains(u64::from(u32::MAX)+1));
    /// assert!(treemap.contains(u64::from(u32::MAX)+5));
    /// assert_eq!(treemap.cardinality(), 11);
    /// ```
    fn from_iter<I: IntoIterator<Item = u64>>(iter: I) -> Self {
        let mut result = Self::new();
        result.extend(iter);
        result
    }
}

impl Extend<u64> for Treemap {
    fn extend<T: IntoIterator<Item = u64>>(&mut self, iter: T) {
        // Potentially reduce outer map lookups by optimistically
        // assuming that adjacent values will belong to the same inner bitmap.
        let mut last_bitmap: Option<(u32, &mut Bitmap)> = None;
        for item in iter {
            let (high, low) = util::split(item);
            if let Some((last_high, ref mut last_bitmap)) = last_bitmap {
                if last_high == high {
                    last_bitmap.add(low);
                    continue;
                }
            }
            let bitmap = self.get_or_create(high);
            bitmap.add(low);
            last_bitmap = Some((high, bitmap));
        }
    }
}
