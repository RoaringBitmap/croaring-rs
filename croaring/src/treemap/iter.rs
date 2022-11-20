use super::util;
use super::{Bitmap, BitmapIterator, Treemap};
use std::collections::btree_map;
use std::iter::{self, FromIterator};

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

fn to64iter<'a>(t: (&'a u32, &'a Bitmap)) -> To64Iter<'a> {
    To64Iter {
        key: *t.0,
        iterator: t.1.iter(),
    }
}

type InnerIter<'a> = iter::FlatMap<
    btree_map::Iter<'a, u32, Bitmap>,
    To64Iter<'a>,
    fn((&'a u32, &'a Bitmap)) -> To64Iter<'a>,
>;

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
    /// let mut treemap = Treemap::create();
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
    pub fn iter(&self) -> TreemapIterator {
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
        let mut result = Self::create();
        result.extend(iter);
        result
    }
}

impl Extend<u64> for Treemap {
    fn extend<T: IntoIterator<Item = u64>>(&mut self, iter: T) {
        for item in iter {
            self.add(item);
        }
    }
}
