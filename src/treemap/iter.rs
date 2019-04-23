use std::iter::FromIterator;

use super::Treemap;

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
        Treemap::of(&Vec::from_iter(iter))
    }
}
