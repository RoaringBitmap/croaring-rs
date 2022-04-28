use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};

use super::Treemap;

impl fmt::Debug for Treemap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.cardinality() < 32 {
            write!(f, "Treemap<{:?}>", self.to_vec())
        } else {
            write!(
                f,
                "Treemap<{}, [{:?}..{:?}]>",
                self.cardinality(),
                self.minimum().unwrap(),
                self.maximum().unwrap()
            )
        }
    }
}

impl Default for Treemap {
    fn default() -> Self {
        Self::create()
    }
}

impl BitAnd for Treemap {
    type Output = Treemap;

    /// Syntactic sugar for `.and`
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
    /// treemap2.add(1);
    /// treemap2.add(u64::MAX);
    ///
    /// let treemap3 = treemap1 & treemap2;
    ///
    /// assert!(treemap3.contains(u64::MAX));
    /// assert!(!treemap3.contains(1));
    /// ```
    #[inline]
    fn bitand(self, other: Treemap) -> Treemap {
        self.and(&other)
    }
}

impl<'a> BitAnd<&'a Treemap> for Treemap {
    type Output = Treemap;

    /// Syntactic sugar for `.and`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Treemap;
    ///
    /// let mut treemap1 = Treemap::create();
    /// treemap1.add(1);
    ///
    /// let mut treemap2 = Treemap::create();
    /// treemap2.add(1);
    /// treemap2.add(2);
    ///
    /// let treemap3 = treemap1 & &treemap2;
    ///
    /// assert!(treemap3.contains(1));
    /// assert!(!treemap3.contains(2));
    /// ```
    #[inline]
    fn bitand(self, other: &'a Treemap) -> Treemap {
        self.and(other)
    }
}

impl<'a, 'b> BitAnd<&'a Treemap> for &'b Treemap {
    type Output = Treemap;

    /// Syntactic sugar for `.and`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Treemap;
    ///
    /// let mut treemap1: Treemap = Treemap::create();
    /// treemap1.add(1);
    ///
    /// let mut treemap2 = Treemap::create();
    /// treemap2.add(1);
    /// treemap2.add(2);
    ///
    /// let treemap3 = &treemap1 & &treemap2;
    ///
    /// assert!(treemap3.contains(1));
    /// assert!(!treemap3.contains(2));
    /// ```
    #[inline]
    fn bitand(self, other: &'a Treemap) -> Treemap {
        self.and(other)
    }
}

impl BitAndAssign for Treemap {
    /// Syntactic sugar for `.and_inplace`
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
    /// let mut treemap4 = Treemap::create();
    /// treemap4.add(15);
    /// treemap4.add(25);
    ///
    /// treemap1 &= treemap2;
    ///
    /// assert!(treemap1.cardinality() == 0);
    /// assert!(!treemap1.contains(15));
    /// assert!(!treemap1.contains(25));
    ///
    /// treemap3 &= treemap4;
    ///
    /// assert!(treemap3.cardinality() == 1);
    /// assert!(treemap3.contains(15));
    /// assert!(!treemap3.contains(25));
    /// ```
    #[inline]
    fn bitand_assign(&mut self, other: Treemap) {
        self.and_inplace(&other);
    }
}

impl BitOr for Treemap {
    type Output = Treemap;

    /// Syntatic sugar for `.or`
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
    /// let treemap3 = treemap1 | treemap2;
    ///
    /// assert!(treemap3.cardinality() == 2);
    /// assert!(treemap3.contains(15));
    /// assert!(treemap3.contains(25));
    /// ```
    #[inline]
    fn bitor(self, other: Treemap) -> Treemap {
        self.or(&other)
    }
}

impl<'a> BitOr<&'a Treemap> for Treemap {
    type Output = Treemap;

    /// Syntatic sugar for `.or`
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
    /// let treemap3 = treemap1 | &treemap2;
    ///
    /// assert!(treemap3.cardinality() == 2);
    /// assert!(treemap3.contains(15));
    /// assert!(treemap3.contains(25));
    /// ```
    #[inline]
    fn bitor(self, other: &'a Treemap) -> Treemap {
        self.or(other)
    }
}

impl<'a, 'b> BitOr<&'a Treemap> for &'b Treemap {
    type Output = Treemap;

    /// Syntatic sugar for `.or`
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
    /// let treemap3 = &treemap1 | &treemap2;
    ///
    /// assert!(treemap3.cardinality() == 2);
    /// assert!(treemap3.contains(15));
    /// assert!(treemap3.contains(25));
    /// ```
    #[inline]
    fn bitor(self, other: &'a Treemap) -> Treemap {
        self.or(other)
    }
}

impl BitOrAssign for Treemap {
    /// Syntatic sugar for `.or_inplace`
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
    /// treemap1 |= treemap2;
    ///
    /// assert!(treemap1.cardinality() == 2);
    /// assert!(treemap1.contains(15));
    /// assert!(treemap1.contains(25));
    /// ```
    #[inline]
    fn bitor_assign(&mut self, other: Treemap) {
        self.or_inplace(&other)
    }
}

impl BitXor for Treemap {
    type Output = Treemap;

    /// Syntatic sugar for `.xor`
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
    /// let treemap3 = treemap1 ^ treemap2;
    ///
    /// assert!(treemap3.cardinality() == 2);
    /// assert!(treemap3.contains(15));
    /// assert!(!treemap3.contains(u64::MAX));
    /// assert!(treemap3.contains(35));
    /// ```
    #[inline]
    fn bitxor(self, other: Treemap) -> Treemap {
        self.xor(&other)
    }
}

impl<'a> BitXor<&'a Treemap> for Treemap {
    type Output = Treemap;

    /// Syntatic sugar for `.xor`
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
    /// let treemap3 = treemap1 ^ &treemap2;
    ///
    /// assert!(treemap3.cardinality() == 2);
    /// assert!(treemap3.contains(15));
    /// assert!(!treemap3.contains(u64::MAX));
    /// assert!(treemap3.contains(35));
    /// ```
    #[inline]
    fn bitxor(self, other: &'a Treemap) -> Treemap {
        self.xor(other)
    }
}

impl<'a, 'b> BitXor<&'a Treemap> for &'b Treemap {
    type Output = Treemap;

    /// Syntatic sugar for `.xor`
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
    /// let treemap3 = &treemap1 ^ &treemap2;
    ///
    /// assert!(treemap3.cardinality() == 2);
    /// assert!(treemap3.contains(15));
    /// assert!(!treemap3.contains(u64::MAX));
    /// assert!(treemap3.contains(35));
    /// ```
    #[inline]
    fn bitxor(self, other: &'a Treemap) -> Treemap {
        self.xor(other)
    }
}

impl BitXorAssign for Treemap {
    /// Syntatic sugar for `.xor_inplace`
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
    /// treemap1 ^= treemap2;
    ///
    /// assert!(treemap1.cardinality() == 2);
    /// assert!(treemap1.contains(15));
    /// assert!(!treemap1.contains(u64::MAX));
    /// assert!(treemap1.contains(35));
    /// ```
    #[inline]
    fn bitxor_assign(&mut self, other: Treemap) {
        self.xor_inplace(&other)
    }
}

impl Sub for Treemap {
    type Output = Treemap;

    /// Syntatic sugar for `.andnot`
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
    /// let treemap3 = treemap1 - treemap2;
    ///
    /// assert_eq!(treemap3.cardinality(), 1);
    /// assert!(treemap3.contains(15));
    /// assert!(!treemap3.contains(u64::MAX));
    /// assert!(!treemap3.contains(35));
    /// ```
    #[inline]
    fn sub(self, other: Treemap) -> Treemap {
        self.andnot(&other)
    }
}

impl<'a> Sub<&'a Treemap> for Treemap {
    type Output = Treemap;

    /// Syntatic sugar for `.andnot`
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
    /// treemap2.add(25);
    /// treemap2.add(u64::MAX);
    ///
    /// let treemap3 = treemap1 - &treemap2;
    ///
    /// assert_eq!(treemap3.cardinality(), 1);
    /// assert!(treemap3.contains(15));
    /// assert!(!treemap3.contains(u64::MAX));
    /// assert!(!treemap3.contains(35));
    /// ```
    #[inline]
    fn sub(self, other: &'a Treemap) -> Treemap {
        self.andnot(other)
    }
}

impl<'a, 'b> Sub<&'a Treemap> for &'b Treemap {
    type Output = Treemap;

    /// Syntatic sugar for `.andnot`
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
    /// treemap2.add(25);
    /// treemap1.add(u64::MAX);
    ///
    /// let treemap3 = &treemap1 - &treemap2;
    ///
    /// assert_eq!(treemap3.cardinality(), 1);
    /// assert!(treemap3.contains(15));
    /// assert!(!treemap3.contains(u64::MAX));
    /// assert!(!treemap3.contains(35));
    /// ```
    #[inline]
    fn sub(self, other: &'a Treemap) -> Treemap {
        self.andnot(other)
    }
}

impl SubAssign for Treemap {
    /// Syntatic sugar for `.andnot_inplace`
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
    /// treemap2.add(25);
    /// treemap2.add(u64::MAX);
    ///
    /// treemap1.andnot_inplace(&treemap2);
    ///
    /// assert_eq!(treemap1.cardinality(), 1);
    /// assert!(treemap1.contains(15));
    /// assert!(!treemap1.contains(u64::MAX));
    /// assert!(!treemap1.contains(35));
    /// ```
    #[inline]
    fn sub_assign(&mut self, other: Treemap) {
        self.andnot_inplace(&other)
    }
}
