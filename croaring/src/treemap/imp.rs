use crate::Bitmap;
use crate::Treemap;

use super::util;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::ops::{Bound, RangeBounds};
use std::u64;

impl Treemap {
    /// Creates an empty `Treemap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use croaring::Treemap;
    /// let treemap = Treemap::create();
    /// ```
    pub fn create() -> Self {
        Treemap {
            map: BTreeMap::new(),
        }
    }

    /// # Examples
    ///
    /// ```rust
    /// use std::u32;
    /// use croaring::Treemap;
    ///
    /// let mut treemap = Treemap::create();
    /// treemap.add(3);
    /// assert!(treemap.contains(3));
    /// treemap.add(u32::MAX as u64);
    /// assert!(treemap.contains(u32::MAX as u64));
    /// treemap.add(u64::from(u32::MAX) + 1);
    /// assert!(treemap.contains(u64::from(u32::MAX)+ 1));
    /// ```
    pub fn add(&mut self, value: u64) {
        let (hi, lo) = util::split(value);
        self.map.entry(hi).or_insert_with(Bitmap::create).add(lo)
    }

    /// Add all values in range
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Treemap;
    ///
    /// let mut treemap1 = Treemap::create();
    /// treemap1.add_range((1..3));
    ///
    /// assert!(!treemap1.is_empty());
    /// assert!(treemap1.contains(1));
    /// assert!(treemap1.contains(2));
    /// assert!(!treemap1.contains(3));
    ///
    /// let mut treemap2 = Treemap::create();
    /// treemap2.add_range((3..1));
    /// assert!(treemap2.is_empty());
    ///
    /// let mut treemap3 = Treemap::create();
    /// treemap3.add_range((3..3));
    /// assert!(treemap3.is_empty());
    ///
    /// let mut treemap4 = Treemap::create();
    /// treemap4.add_range(..=2);
    /// treemap4.add_range(u64::MAX..=u64::MAX);
    /// assert!(treemap4.contains(0));
    /// assert!(treemap4.contains(1));
    /// assert!(treemap4.contains(2));
    /// assert!(treemap4.contains(u64::MAX));
    /// assert_eq!(treemap4.cardinality(), 4);
    /// ```
    pub fn add_range<R: RangeBounds<u64>>(&mut self, range: R) {
        let (start, end) = range_to_inclusive(range);
        self.add_range_inclusive(start, end);
    }

    fn add_range_inclusive(&mut self, start: u64, end: u64) {
        if start > end {
            return;
        }
        let (start_high, start_low) = util::split(start);
        let (end_high, end_low) = util::split(end);
        if start_high == end_high {
            self.map
                .entry(start_high)
                .or_default()
                .add_range(start_low..=end_low);
            return;
        }

        // Because start and end don't land on the same inner bitmap,
        // we need to do this in multiple steps:
        // 1. Partially fill the first bitmap with values from the closed
        //    interval [start_low, uint32_max]
        // 2. Fill intermediate bitmaps completely: [0, uint32_max]
        // 3. Partially fill the last bitmap with values from the closed
        //    interval [0, end_low]

        // Step 1: Partially fill the first bitmap
        {
            let bitmap = self.map.entry(start_high).or_insert_with(Bitmap::create);
            bitmap.add_range(start_low..=u32::MAX);
        }
        // Step 2: Fill intermediate bitmaps completely
        for i in start_high + 1..end_high {
            // This blows away the container, is it worth trying to save any existing alocations?
            self.map.insert(i, Bitmap::from_range(0..=u32::MAX));
        }
        // Step 3: Partially fill the last bitmap
        {
            let bitmap = self.map.entry(end_high).or_insert_with(Bitmap::create);
            bitmap.add_range(0..=end_low);
        }
    }

    /// ```rust
    /// use croaring::Treemap;
    ///
    /// let mut treemap = Treemap::create();
    /// ```
    pub fn contains(&self, value: u64) -> bool {
        let (hi, lo) = util::split(value);
        match self.map.get(&hi) {
            None => false,
            Some(r) => r.contains(lo),
        }
    }

    /// Returns true if the Treemap is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::u64;
    /// use croaring::Treemap;
    ///
    /// let mut treemap = Treemap::create();
    ///
    /// assert!(treemap.is_empty());
    ///
    /// treemap.add(u64::MAX);
    ///
    /// assert!(!treemap.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.map.values().all(Bitmap::is_empty)
    }

    /// Empties the Treemap
    ///
    /// # Examples
    ///
    /// ```
    /// use std::u64;
    /// use croaring::Treemap;
    ///
    /// let mut treemap = Treemap::create();
    ///
    /// treemap.add(1);
    /// treemap.add(u64::MAX);
    ///
    /// assert!(!treemap.is_empty());
    ///
    /// treemap.clear();
    ///
    /// assert!(treemap.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.map.iter_mut().for_each(|(_, bitmap)| bitmap.clear())
    }

    /// Remove element from the Treemap
    ///
    /// # Examples
    ///
    /// ```
    /// use std::u64;
    /// use croaring::Treemap;
    ///
    /// let mut treemap = Treemap::create();
    /// treemap.add(u64::MAX);
    /// treemap.remove(u64::MAX);
    ///
    /// assert!(treemap.is_empty());
    /// ```
    pub fn remove(&mut self, element: u64) {
        let (hi, lo) = util::split(element);
        match self.map.entry(hi) {
            Entry::Vacant(_) => (),
            Entry::Occupied(mut bitmap) => {
                bitmap.get_mut().remove(lo);
                if bitmap.get().is_empty() {
                    bitmap.remove();
                }
            }
        }
    }

    /// Returns the number of elements contained in the Treemap
    ///
    /// # Examples
    ///
    /// ```
    /// use std::u64;
    /// use croaring::Treemap;
    ///
    /// let mut treemap = Treemap::create();
    /// treemap.add(1);
    ///
    /// assert_eq!(treemap.cardinality(), 1);
    ///
    /// treemap.add(u64::MAX);
    ///
    /// assert_eq!(treemap.cardinality(), 2);
    /// ```
    pub fn cardinality(&self) -> u64 {
        self.map.values().map(Bitmap::cardinality).sum()
    }

    /// Returns the smallest value in the set.
    /// Returns std::u64::MAX if the set is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Treemap;
    ///
    /// let mut treemap: Treemap = Treemap::create();
    /// let empty_treemap: Treemap = Treemap::create();
    ///
    /// treemap.add(120);
    /// treemap.add(1000);
    ///
    /// assert_eq!(treemap.minimum(), Some(120));
    /// assert_eq!(empty_treemap.minimum(), None);
    /// ```
    pub fn minimum(&self) -> Option<u64> {
        self.map
            .iter()
            .filter(|(_, bitmap)| !bitmap.is_empty())
            .map(|(k, bitmap)| util::join(*k, bitmap.minimum().unwrap()))
            .next()
    }

    /// Returns the greatest value in the set.
    /// Returns 0 if the set is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Treemap;
    ///
    /// let mut treemap: Treemap = Treemap::create();
    /// let empty_treemap: Treemap = Treemap::create();
    ///
    /// treemap.add(120);
    /// treemap.add(1000);
    ///
    /// assert_eq!(treemap.maximum(), Some(1000));
    /// assert_eq!(empty_treemap.maximum(), None);
    /// ```
    pub fn maximum(&self) -> Option<u64> {
        self.map
            .iter()
            .rev()
            .filter(|(_, bitmap)| !bitmap.is_empty())
            .map(|(k, bitmap)| util::join(*k, bitmap.maximum().unwrap()))
            .next()
    }

    /// And computes the intersection between two treemaps and returns the
    /// result as a new treemap
    ///
    /// # Examples
    ///
    /// ```
    /// use std::u64;
    /// use croaring::Treemap;
    ///
    /// let mut treemap1 = Treemap::create();
    /// treemap1.add(u64::MAX);
    ///
    /// let mut treemap2 = Treemap::create();
    /// treemap2.add(u64::MAX);
    /// treemap2.add(2);
    ///
    /// let treemap3 = treemap1.and(&treemap2);
    ///
    /// assert!(treemap3.contains(u64::MAX));
    /// assert!(!treemap3.contains(2));
    /// ```
    pub fn and(&self, other: &Self) -> Self {
        let mut treemap = Treemap::create();

        for (key, bitmap) in &self.map {
            other
                .map
                .get(key)
                .map(|other_bitmap| treemap.map.insert(*key, bitmap.and(other_bitmap)));
        }

        treemap
    }

    /// Computes the intersection between two treemaps and stores the result
    /// in the current treemap
    ///
    /// # Examples
    ///
    /// ```
    /// use std::u64;
    /// use croaring::Treemap;
    ///
    /// let mut treemap1 = Treemap::create();
    /// treemap1.add(u64::MAX);
    ///
    /// let mut treemap2 = Treemap::create();
    /// treemap2.add(25);
    ///
    /// let mut treemap3 = Treemap::create();
    /// treemap3.add(u64::MAX);
    ///
    /// let mut treemap4 = Treemap::create();
    /// treemap4.add(u64::MAX);
    /// treemap4.add(25);
    ///
    /// treemap1.and_inplace(&treemap2);
    ///
    /// assert_eq!(treemap1.cardinality(), 0);
    /// assert!(!treemap1.contains(u64::MAX));
    /// assert!(!treemap1.contains(25));
    ///
    /// treemap3.and_inplace(&treemap4);
    ///
    /// assert_eq!(treemap3.cardinality(), 1);
    /// assert!(treemap3.contains(u64::MAX));
    /// assert!(!treemap3.contains(25));
    ///
    /// let mut treemap5 = Treemap::create();
    /// treemap5.add(u64::MAX);
    /// treemap5.and_inplace(&Treemap::create());
    /// assert_eq!(treemap5.cardinality(), 0);
    /// ```
    pub fn and_inplace(&mut self, other: &Self) {
        let mut keys_to_remove: Vec<u32> = Vec::new();

        for (key, bitmap) in &mut self.map {
            match other.map.get(key) {
                None => {
                    keys_to_remove.push(*key);
                }
                Some(other_bitmap) => {
                    bitmap.and_inplace(other_bitmap);
                    if bitmap.is_empty() {
                        keys_to_remove.push(*key);
                    }
                }
            }
        }

        for key in keys_to_remove {
            self.map.remove(&key);
        }
    }

    /// Or computes the union between two bitmaps and returns the result
    /// as a new bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use std::u64;
    /// use croaring::Treemap;
    ///
    /// let mut treemap1 = Treemap::create();
    /// treemap1.add(u64::MAX);
    ///
    /// let mut treemap2 = Treemap::create();
    /// treemap2.add(25);
    ///
    /// let treemap3 = treemap1.or(&treemap2);
    ///
    /// assert!(treemap3.cardinality() == 2);
    /// assert!(treemap3.contains(u64::MAX));
    /// assert!(treemap3.contains(25));
    /// ```
    pub fn or(&self, other: &Self) -> Self {
        let mut treemap = self.clone();

        for (key, other_bitmap) in &other.map {
            match treemap.map.entry(*key) {
                Entry::Vacant(current_map) => {
                    current_map.insert(other_bitmap.clone());
                }
                Entry::Occupied(mut bitmap) => {
                    bitmap.get_mut().or_inplace(other_bitmap);
                }
            };
        }

        treemap
    }

    /// Computes the intersection between two bitmaps and stores the result
    /// in the current bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Treemap;
    ///
    /// let mut treemap1 = Treemap::create();
    /// treemap1.add(15);
    ///
    /// let mut treemap2 = Treemap::create();
    /// treemap2.add(25);
    ///
    /// let mut treemap3 = Treemap::create();
    /// treemap3.add(15);
    ///
    /// let mut bitmap4 = Treemap::create();
    /// bitmap4.add(15);
    /// bitmap4.add(25);
    ///
    /// treemap1.and_inplace(&treemap2);
    ///
    /// assert_eq!(treemap1.cardinality(), 0);
    /// assert!(!treemap1.contains(15));
    /// assert!(!treemap1.contains(25));
    ///
    /// treemap3.and_inplace(&bitmap4);
    ///
    /// assert_eq!(treemap3.cardinality(), 1);
    /// assert!(treemap3.contains(15));
    /// assert!(!treemap3.contains(25));
    /// ```
    pub fn or_inplace(&mut self, other: &Self) {
        for (key, other_bitmap) in &other.map {
            match self.map.entry(*key) {
                Entry::Vacant(current_map) => {
                    current_map.insert(other_bitmap.clone());
                }
                Entry::Occupied(mut current_map) => {
                    current_map.get_mut().or_inplace(other_bitmap);
                }
            };
        }
    }

    /// Computes the symmetric difference (xor) between two treemaps
    /// and returns a new treemap.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::u64;
    /// use croaring::Treemap;
    ///
    /// let mut treemap1 = Treemap::create();
    /// treemap1.add(15);
    /// treemap1.add(u64::MAX);
    ///
    /// let mut treemap2 = Treemap::create();
    /// treemap2.add(u64::MAX);
    /// treemap2.add(35);
    ///
    /// let treemap3 = treemap1.xor(&treemap2);
    ///
    /// assert_eq!(treemap3.cardinality(), 2);
    /// assert!(treemap3.contains(15));
    /// assert!(!treemap3.contains(25));
    /// assert!(treemap3.contains(35));
    /// ```
    pub fn xor(&self, other: &Self) -> Self {
        let mut treemap = self.clone();

        for (key, other_bitmap) in &other.map {
            match treemap.map.entry(*key) {
                Entry::Vacant(current_map) => {
                    current_map.insert(other_bitmap.clone());
                }
                Entry::Occupied(mut bitmap) => {
                    bitmap.get_mut().xor_inplace(other_bitmap);
                }
            };
        }

        treemap
    }

    /// Inplace version of xor, stores result in the current treemap.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::u64;
    /// use croaring::Treemap;
    ///
    /// let mut treemap1 = Treemap::create();
    /// treemap1.add(15);
    /// treemap1.add(25);
    ///
    /// let mut treemap2 = Treemap::create();
    /// treemap2.add(25);
    /// treemap2.add(35);
    ///
    /// treemap1.xor_inplace(&treemap2);
    ///
    /// assert_eq!(treemap1.cardinality(), 2);
    /// assert!(treemap1.contains(15));
    /// assert!(treemap1.contains(35));
    ///
    /// let mut treemap3 = Treemap::create();
    /// treemap3.add(15);
    /// treemap3.xor_inplace(&Treemap::create());
    /// assert_eq!(treemap3.cardinality(), 1);
    /// assert!(treemap3.contains(15));
    /// ```
    pub fn xor_inplace(&mut self, other: &Self) {
        let mut keys_to_remove: Vec<u32> = Vec::new();

        for (key, other_bitmap) in &other.map {
            match self.map.entry(*key) {
                Entry::Vacant(bitmap) => {
                    bitmap.insert(other_bitmap.clone());
                }
                Entry::Occupied(mut bitmap) => {
                    bitmap.get_mut().xor_inplace(other_bitmap);
                    if bitmap.get().is_empty() {
                        keys_to_remove.push(*key);
                    }
                }
            };
        }

        for key in keys_to_remove {
            self.map.remove(&key);
        }
    }

    /// Computes the difference between two bitmaps and returns the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::u64;
    /// use croaring::Treemap;
    ///
    /// let mut treemap1 = Treemap::create();
    ///
    /// treemap1.add(15);
    /// treemap1.add(u64::MAX);
    ///
    /// let mut treemap2 = Treemap::create();
    ///
    /// treemap2.add(u64::MAX);
    /// treemap2.add(35);
    ///
    /// let treemap3 = treemap1.andnot(&treemap2);
    ///
    /// assert_eq!(treemap3.cardinality(), 1);
    /// assert!(treemap3.contains(15));
    /// assert!(!treemap3.contains(u64::MAX));
    /// assert!(!treemap3.contains(35));
    /// ```
    pub fn andnot(&self, other: &Self) -> Self {
        let mut treemap = Treemap::create();

        for (key, bitmap) in &self.map {
            if let Some(other_bitmap) = other.map.get(key) {
                treemap.map.insert(*key, bitmap.andnot(other_bitmap));
            }
        }

        treemap
    }

    /// Computes the difference between two treemaps and stores the result
    /// in the current treemap.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::u32;
    /// use std::u64;
    /// use croaring::Treemap;
    ///
    /// let mut treemap1 = Treemap::create();
    ///
    /// treemap1.add(15);
    /// treemap1.add(25);
    /// treemap1.add(u64::MAX - 10);
    ///
    /// let mut treemap2 = Treemap::create();
    ///
    /// treemap2.add(25);
    /// treemap2.add(35);
    ///
    /// treemap1.andnot_inplace(&treemap2);
    ///
    /// assert_eq!(treemap1.cardinality(), 2);
    /// assert!(treemap1.contains(15));
    /// assert!(treemap1.contains(u64::MAX - 10));
    /// assert!(!treemap1.contains(u64::MAX));
    /// assert!(!treemap1.contains(35));
    ///
    /// let mut treemap3 = Treemap::create();
    /// treemap3.add(15);
    /// let treemap4 = Treemap::create();
    /// treemap3.andnot_inplace(&treemap4);
    /// assert_eq!(treemap3.cardinality(), 1);
    /// assert!(treemap3.contains(15));
    /// ```
    pub fn andnot_inplace(&mut self, other: &Self) {
        for (key, bitmap) in &mut self.map {
            if let Some(other_bitmap) = other.map.get(key) {
                bitmap.andnot_inplace(other_bitmap);
            }
        }
    }

    /// Returns a vector containing all of the integers stored in the Treemap
    /// in a sorted order.
    ///
    /// ```
    /// use std::u64;
    /// use croaring::Treemap;
    ///
    /// let mut treemap = Treemap::create();
    /// treemap.add(25);
    /// treemap.add(15);
    /// treemap.add(u64::MAX);
    ///
    /// assert_eq!(treemap.to_vec(), [15, 25, u64::MAX]);
    /// ```
    pub fn to_vec(&self) -> Vec<u64> {
        let treemap_size: usize = self.cardinality().try_into().unwrap();

        let mut result: Vec<u64> = Vec::with_capacity(treemap_size);
        let mut buffer = [0; 1024];

        for (&key, bitmap) in &self.map {
            let mut iter = bitmap.iter();
            loop {
                let n = iter.next_many(&mut buffer);
                if n == 0 {
                    break;
                }
                result.extend(buffer[..n].iter().map(|&bit| util::join(key, bit)))
            }
        }

        result
    }

    /// Creates a new treemap from a slice of u64 integers
    ///
    /// # Examples
    ///
    /// ```
    /// use std::u64;
    /// use croaring::Treemap;
    ///
    /// let elements = vec![1, 2, u64::MAX];
    ///
    /// let treemap = Treemap::of(&elements);
    ///
    /// let mut treemap2 = Treemap::create();
    ///
    /// for element in &elements {
    ///     treemap2.add(*element);
    /// }
    ///
    /// assert!(treemap.contains(1));
    /// assert!(treemap.contains(2));
    /// assert!(treemap.contains(u64::MAX));
    /// assert!(!treemap.contains(3));
    /// assert_eq!(treemap, treemap2);
    /// ```
    pub fn of(elements: &[u64]) -> Self {
        let mut treemap = Treemap::create();

        for element in elements {
            treemap.add(*element);
        }

        treemap
    }

    /// Compresses treemap's bitmaps. Returns true if any of the bitmaps
    /// were modified.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Treemap;
    ///
    /// let mut treemap: Treemap = (100..1000).collect();
    ///
    /// assert_eq!(treemap.cardinality(), 900);
    /// assert!(treemap.run_optimize());
    /// ```
    pub fn run_optimize(&mut self) -> bool {
        self.map
            .iter_mut()
            .fold(false, |result, (_, bitmap)| bitmap.run_optimize() || result)
    }

    /// Removes run-length encoding from treemap's bitmaps. Returns true if
    /// change was made to any of the bitmaps.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Treemap;
    ///
    /// let mut treemap: Treemap = (100..1000).collect();
    ///
    /// assert_eq!(treemap.cardinality(), 900);
    /// assert!(treemap.run_optimize());
    /// assert!(treemap.remove_run_compression());
    /// ```
    pub fn remove_run_compression(&mut self) -> bool {
        self.map.iter_mut().fold(false, |result, (_, bitmap)| {
            bitmap.remove_run_compression() || result
        })
    }

    /// Return true if all the elements of Self are in &other.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Treemap;
    ///
    /// let bitmap1: Treemap = (5..10).collect();
    /// let bitmap2: Treemap = (5..8).collect();
    /// let bitmap3: Treemap = (5..10).collect();
    /// let bitmap4: Treemap = (9..11).collect();
    ///
    /// assert!(bitmap2.is_subset(&bitmap1));
    /// assert!(bitmap3.is_subset(&bitmap1));
    /// assert!(!bitmap4.is_subset(&bitmap1));
    /// ```
    pub fn is_subset(&self, other: &Self) -> bool {
        for (k, v) in self.map.iter() {
            if v.is_empty() {
                continue;
            }
            match other.map.get(k) {
                None => return false,
                Some(other_v) => {
                    if !v.is_subset(other_v) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Return true if all the elements of Self are in &other and &other is strictly greater
    /// than Self.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Treemap;
    ///
    /// let bitmap1: Treemap = (5..9).collect();
    /// let bitmap2: Treemap = (5..8).collect();
    /// let bitmap3: Treemap = (5..10).collect();
    /// let bitmap4: Treemap = (9..11).collect();
    ///
    /// assert!(bitmap2.is_subset(&bitmap1));
    /// assert!(!bitmap3.is_subset(&bitmap1));
    /// assert!(!bitmap4.is_subset(&bitmap1));
    /// ```
    pub fn is_strict_subset(&self, other: &Self) -> bool {
        self.is_subset(other) && self.cardinality() != other.cardinality()
    }
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
