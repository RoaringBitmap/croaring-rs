use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};

use super::{ffi, Bitmap};

impl fmt::Debug for Bitmap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.cardinality() < 32 {
            write!(f, "Bitmap<{:?}>", self.to_vec())
        } else {
            write!(
                f,
                "Bitmap<{:?} values between {:?} and {:?}>",
                self.cardinality(),
                self.minimum().unwrap(),
                self.maximum().unwrap()
            )
        }
    }
}

impl PartialEq for Bitmap {
    #[inline]
    fn eq(&self, other: &Bitmap) -> bool {
        unsafe { ffi::roaring_bitmap_equals(self.bitmap, other.bitmap) }
    }
}

impl Clone for Bitmap {
    /// Create a copy of a Bitmap
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(11);
    ///
    /// let bitmap2 = bitmap1.clone();
    ///
    /// assert_eq!(bitmap1, bitmap2);
    /// ```
    #[inline]
    fn clone(&self) -> Bitmap {
        unsafe {
            Bitmap {
                bitmap: ffi::roaring_bitmap_copy(self.bitmap),
            }
        }
    }
}

impl Drop for Bitmap {
    fn drop(&mut self) {
        unsafe { ffi::roaring_bitmap_free(self.bitmap) }
    }
}

impl BitAnd for Bitmap {
    type Output = Bitmap;

    /// Syntactic sugar for `.and`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(1);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(1);
    /// bitmap2.add(2);
    ///
    /// let bitmap3 = bitmap1 & bitmap2;
    ///
    /// assert!(bitmap3.contains(1));
    /// assert!(!bitmap3.contains(2));
    /// ```
    #[inline]
    fn bitand(self, other: Bitmap) -> Bitmap {
        self.and(&other)
    }
}

impl<'a> BitAnd<&'a Bitmap> for Bitmap {
    type Output = Bitmap;

    /// Syntactic sugar for `.and`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(1);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(1);
    /// bitmap2.add(2);
    ///
    /// let bitmap3 = bitmap1 & &bitmap2;
    ///
    /// assert!(bitmap3.contains(1));
    /// assert!(!bitmap3.contains(2));
    /// ```
    #[inline]
    fn bitand(self, other: &'a Bitmap) -> Bitmap {
        self.and(&other)
    }
}

impl<'a, 'b> BitAnd<&'a Bitmap> for &'b Bitmap {
    type Output = Bitmap;

    /// Syntactic sugar for `.and`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1: Bitmap = Bitmap::create();
    /// bitmap1.add(1);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(1);
    /// bitmap2.add(2);
    ///
    /// let bitmap3 = &bitmap1 & &bitmap2;
    ///
    /// assert!(bitmap3.contains(1));
    /// assert!(!bitmap3.contains(2));
    /// ```
    #[inline]
    fn bitand(self, other: &'a Bitmap) -> Bitmap {
        self.and(&other)
    }
}

impl BitAndAssign for Bitmap {
    /// Syntactic sugar for `.and_inplace`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(15);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(25);
    ///
    /// let mut bitmap3 = Bitmap::create();
    /// bitmap3.add(15);
    ///
    /// let mut bitmap4 = Bitmap::create();
    /// bitmap4.add(15);
    /// bitmap4.add(25);
    ///
    /// bitmap1 &= bitmap2;
    ///
    /// assert!(bitmap1.cardinality() == 0);
    /// assert!(!bitmap1.contains(15));
    /// assert!(!bitmap1.contains(25));
    ///
    /// bitmap3 &= bitmap4;
    ///
    /// assert!(bitmap3.cardinality() == 1);
    /// assert!(bitmap3.contains(15));
    /// assert!(!bitmap3.contains(25));
    /// ```
    #[inline]
    fn bitand_assign(&mut self, other: Bitmap) {
        self.and_inplace(&other);
    }
}

impl BitOr for Bitmap {
    type Output = Bitmap;

    /// Syntatic sugar for `.or`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(15);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(25);
    ///
    /// let bitmap3 = bitmap1 | bitmap2;
    ///
    /// assert!(bitmap3.cardinality() == 2);
    /// assert!(bitmap3.contains(15));
    /// assert!(bitmap3.contains(25));
    /// ```
    #[inline]
    fn bitor(self, other: Bitmap) -> Bitmap {
        self.or(&other)
    }
}

impl<'a> BitOr<&'a Bitmap> for Bitmap {
    type Output = Bitmap;

    /// Syntatic sugar for `.or`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(15);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(25);
    ///
    /// let bitmap3 = bitmap1 | &bitmap2;
    ///
    /// assert!(bitmap3.cardinality() == 2);
    /// assert!(bitmap3.contains(15));
    /// assert!(bitmap3.contains(25));
    /// ```
    #[inline]
    fn bitor(self, other: &'a Bitmap) -> Bitmap {
        self.or(&other)
    }
}

impl<'a, 'b> BitOr<&'a Bitmap> for &'b Bitmap {
    type Output = Bitmap;

    /// Syntatic sugar for `.or`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(15);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(25);
    ///
    /// let bitmap3 = &bitmap1 | &bitmap2;
    ///
    /// assert!(bitmap3.cardinality() == 2);
    /// assert!(bitmap3.contains(15));
    /// assert!(bitmap3.contains(25));
    /// ```
    #[inline]
    fn bitor(self, other: &'a Bitmap) -> Bitmap {
        self.or(&other)
    }
}

impl BitOrAssign for Bitmap {
    /// Syntatic sugar for `.or_inplace`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(15);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(25);
    ///
    /// bitmap1 |= bitmap2;
    ///
    /// assert!(bitmap1.cardinality() == 2);
    /// assert!(bitmap1.contains(15));
    /// assert!(bitmap1.contains(25));
    /// ```
    #[inline]
    fn bitor_assign(&mut self, other: Bitmap) {
        self.or_inplace(&other)
    }
}

impl BitXor for Bitmap {
    type Output = Bitmap;

    /// Syntatic sugar for `.xor`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(15);
    /// bitmap1.add(25);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(25);
    /// bitmap2.add(35);
    ///
    /// let bitmap3 = bitmap1 ^ bitmap2;
    ///
    /// assert!(bitmap3.cardinality() == 2);
    /// assert!(bitmap3.contains(15));
    /// assert!(!bitmap3.contains(25));
    /// assert!(bitmap3.contains(35));
    /// ```
    #[inline]
    fn bitxor(self, other: Bitmap) -> Bitmap {
        self.xor(&other)
    }
}

impl<'a> BitXor<&'a Bitmap> for Bitmap {
    type Output = Bitmap;

    /// Syntatic sugar for `.xor`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(15);
    /// bitmap1.add(25);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(25);
    /// bitmap2.add(35);
    ///
    /// let bitmap3 = bitmap1 ^ &bitmap2;
    ///
    /// assert!(bitmap3.cardinality() == 2);
    /// assert!(bitmap3.contains(15));
    /// assert!(!bitmap3.contains(25));
    /// assert!(bitmap3.contains(35));
    /// ```
    #[inline]
    fn bitxor(self, other: &'a Bitmap) -> Bitmap {
        self.xor(&other)
    }
}

impl<'a, 'b> BitXor<&'a Bitmap> for &'b Bitmap {
    type Output = Bitmap;

    /// Syntatic sugar for `.xor`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(15);
    /// bitmap1.add(25);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(25);
    /// bitmap2.add(35);
    ///
    /// let bitmap3 = &bitmap1 ^ &bitmap2;
    ///
    /// assert!(bitmap3.cardinality() == 2);
    /// assert!(bitmap3.contains(15));
    /// assert!(!bitmap3.contains(25));
    /// assert!(bitmap3.contains(35));
    /// ```
    #[inline]
    fn bitxor(self, other: &'a Bitmap) -> Bitmap {
        self.xor(&other)
    }
}

impl BitXorAssign for Bitmap {
    /// Syntatic sugar for `.xor_inplace`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    /// bitmap1.add(15);
    /// bitmap1.add(25);
    ///
    /// let mut bitmap2 = Bitmap::create();
    /// bitmap2.add(25);
    /// bitmap2.add(35);
    ///
    /// bitmap1 ^= bitmap2;
    ///
    /// assert!(bitmap1.cardinality() == 2);
    /// assert!(bitmap1.contains(15));
    /// assert!(!bitmap1.contains(25));
    /// assert!(bitmap1.contains(35));
    /// ```
    #[inline]
    fn bitxor_assign(&mut self, other: Bitmap) {
        self.xor_inplace(&other)
    }
}

impl Sub for Bitmap {
    type Output = Bitmap;

    /// Syntatic sugar for `.andnot`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    ///
    /// bitmap1.add(15);
    /// bitmap1.add(25);
    ///
    /// let mut bitmap2 = Bitmap::create();
    ///
    /// bitmap2.add(25);
    /// bitmap2.add(35);
    ///
    /// let bitmap3 = bitmap1 - bitmap2;
    ///
    /// assert_eq!(bitmap3.cardinality(), 1);
    /// assert!(bitmap3.contains(15));
    /// assert!(!bitmap3.contains(25));
    /// assert!(!bitmap3.contains(35));
    /// ```
    #[inline]
    fn sub(self, other: Bitmap) -> Bitmap {
        self.andnot(&other)
    }
}

impl<'a> Sub<&'a Bitmap> for Bitmap {
    type Output = Bitmap;

    /// Syntatic sugar for `.andnot`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    ///
    /// bitmap1.add(15);
    /// bitmap1.add(25);
    ///
    /// let mut bitmap2 = Bitmap::create();
    ///
    /// bitmap2.add(25);
    /// bitmap2.add(35);
    ///
    /// let bitmap3 = bitmap1 - &bitmap2;
    ///
    /// assert_eq!(bitmap3.cardinality(), 1);
    /// assert!(bitmap3.contains(15));
    /// assert!(!bitmap3.contains(25));
    /// assert!(!bitmap3.contains(35));
    /// ```
    #[inline]
    fn sub(self, other: &'a Bitmap) -> Bitmap {
        self.andnot(&other)
    }
}

impl<'a, 'b> Sub<&'a Bitmap> for &'b Bitmap {
    type Output = Bitmap;

    /// Syntatic sugar for `.andnot`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    ///
    /// bitmap1.add(15);
    /// bitmap1.add(25);
    ///
    /// let mut bitmap2 = Bitmap::create();
    ///
    /// bitmap2.add(25);
    /// bitmap2.add(35);
    ///
    /// let bitmap3 = &bitmap1 - &bitmap2;
    ///
    /// assert_eq!(bitmap3.cardinality(), 1);
    /// assert!(bitmap3.contains(15));
    /// assert!(!bitmap3.contains(25));
    /// assert!(!bitmap3.contains(35));
    /// ```
    #[inline]
    fn sub(self, other: &'a Bitmap) -> Bitmap {
        self.andnot(&other)
    }
}

impl SubAssign for Bitmap {
    /// Syntatic sugar for `.andnot_inplace`
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::create();
    ///
    /// bitmap1.add(15);
    /// bitmap1.add(25);
    ///
    /// let mut bitmap2 = Bitmap::create();
    ///
    /// bitmap2.add(25);
    /// bitmap2.add(35);
    ///
    /// bitmap1.andnot_inplace(&bitmap2);
    ///
    /// assert_eq!(bitmap1.cardinality(), 1);
    /// assert!(bitmap1.contains(15));
    /// assert!(!bitmap1.contains(25));
    /// assert!(!bitmap1.contains(35));
    /// ```
    #[inline]
    fn sub_assign(&mut self, other: Bitmap) {
        self.andnot_inplace(&other)
    }
}
