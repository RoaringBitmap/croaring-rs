use Bitmap;
use Treemap;

use super::util;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
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
    /// assert_eq!(treemap.minimum(), 120);
    /// assert_eq!(empty_treemap.minimum(), std::u64::MAX);
    /// ```
    pub fn minimum(&self) -> u64 {
        self.map
            .iter()
            .filter(|(_, bitmap)| !bitmap.is_empty())
            .map(|(k, bitmap)| util::join(*k, bitmap.minimum()))
            .next()
            .unwrap_or(u64::MAX)
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
    /// assert_eq!(treemap.maximum(), 1000);
    /// assert_eq!(empty_treemap.maximum(), 0);
    /// ```
    pub fn maximum(&self) -> u64 {
        self.map
            .iter()
            .filter(|(_, bitmap)| !bitmap.is_empty())
            .map(|(k, bitmap)| util::join(*k, bitmap.maximum()))
            .next()
            .unwrap_or(0)
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
                .get(&key)
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
    /// assert!(treemap1.cardinality() == 0);
    /// assert!(!treemap1.contains(u64::MAX));
    /// assert!(!treemap1.contains(25));
    ///
    /// treemap3.and_inplace(&treemap4);
    ///
    /// assert!(treemap3.cardinality() == 1);
    /// assert!(treemap3.contains(u64::MAX));
    /// assert!(!treemap3.contains(25));
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
    /// assert!(treemap1.cardinality() == 0);
    /// assert!(!treemap1.contains(15));
    /// assert!(!treemap1.contains(25));
    ///
    /// treemap3.and_inplace(&bitmap4);
    ///
    /// assert!(treemap3.cardinality() == 1);
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
    /// assert!(treemap3.cardinality() == 2);
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
    /// assert!(treemap1.cardinality() == 2);
    /// assert!(treemap1.contains(15));
    /// assert!(!treemap1.contains(u64::MAX));
    /// assert!(treemap1.contains(35));
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
    /// use std::u64;
    /// use croaring::Treemap;
    ///
    /// let mut treemap1 = Treemap::create();
    ///
    /// treemap1.add(15);
    /// treemap1.add(25);
    ///
    /// let mut treemap2 = Treemap::create();
    ///
    /// treemap2.add(25);
    /// treemap2.add(35);
    ///
    /// treemap1.andnot_inplace(&treemap2);
    ///
    /// assert_eq!(treemap1.cardinality(), 1);
    /// assert!(treemap1.contains(15));
    /// assert!(!treemap1.contains(u64::MAX));
    /// assert!(!treemap1.contains(35));
    /// ```
    pub fn andnot_inplace(&mut self, other: &Self) {
        let mut keys_to_remove: Vec<u32> = Vec::new();

        for (key, bitmap) in &mut self.map {
            if let Some(other_bitmap) = other.map.get(key) {
                bitmap.andnot_inplace(other_bitmap);
            } else {
                keys_to_remove.push(*key);
            }
        }

        for key in keys_to_remove {
            self.map.remove(&key);
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
        let treemap_size = self.cardinality();

        let mut buffer: Vec<u64> = Vec::with_capacity(treemap_size as usize);

        for (key, bitmap) in &self.map {
            bitmap.iter().for_each(|bit| buffer.push(util::join(*key, bit)));
        }

        buffer
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
        self.map.iter_mut().fold(
            false,
            |result, (_, bitmap)| result || bitmap.run_optimize()
        )
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
        self.map.iter_mut().fold(
            false,
            |result, (_, bitmap)| result || bitmap.remove_run_compression()
        )
    }
}
