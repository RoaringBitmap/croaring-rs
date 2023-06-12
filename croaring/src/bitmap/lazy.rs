use crate::Bitmap;

pub struct LazyBitmap<'a> {
    bitmap: &'a mut Bitmap,
}

impl<'a> LazyBitmap<'a> {
    /// Modifies the bitmap this lazy bitmap is associated with to be the union of the two bitmaps.
    ///
    /// # Arguments
    /// * `other` - The other bitmap to union with.
    /// * `force_bitsets` - Whether to force conversions to bitsets when modifying containers
    #[inline]
    #[doc(alias = "roaring_bitmap_lazy_or_inplace")]
    pub fn or_inplace(&mut self, other: &Bitmap, force_bitsets: bool) -> &mut Self {
        unsafe {
            // Because we have a mutable borrow of the bitmap, `other` cannot be == our bitmap,
            // so this is always safe
            ffi::roaring_bitmap_lazy_or_inplace(
                &mut self.bitmap.bitmap,
                &other.bitmap,
                force_bitsets,
            );
        }
        self
    }

    /// Modifies the bitmap this lazy bitmap is associated with to be the xor of the two bitmaps.
    #[inline]
    #[doc(alias = "roaring_bitmap_lazy_xor_inplace")]
    pub fn xor_inplace(&mut self, other: &Bitmap) -> &mut Self {
        unsafe {
            // Because we have a mutable borrow of the bitmap, `other` cannot be == our bitmap,
            // so this is always safe
            ffi::roaring_bitmap_lazy_xor_inplace(&mut self.bitmap.bitmap, &other.bitmap);
        }
        self
    }
}

impl<'a> std::ops::BitOrAssign<&Bitmap> for LazyBitmap<'a> {
    #[inline]
    fn bitor_assign(&mut self, other: &Bitmap) {
        self.or_inplace(other, false);
    }
}

impl<'a> std::ops::BitXorAssign<&Bitmap> for LazyBitmap<'a> {
    #[inline]
    fn bitxor_assign(&mut self, other: &Bitmap) {
        self.xor_inplace(other);
    }
}

impl Bitmap {
    /// Perform multiple bitwise operations on a bitmap.
    ///
    /// The passed closure will be passed a handle which can be used to perform bitwise operations on the bitmap lazily.
    ///
    /// The result will be equivalent to doing the same operations on this bitmap directly, but because of reduced
    /// bookkeeping in between operations, it should be faster
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// // Perform a series of bitwise operations on a bitmap:
    /// let mut bitmap = Bitmap::of(&[99]);
    /// let bitmaps_to_or = [Bitmap::of(&[1, 2, 5, 10]), Bitmap::of(&[1, 30, 100])];
    /// let bitmaps_to_xor = [Bitmap::of(&[5]), Bitmap::of(&[1, 1000, 1001])];
    ///
    /// bitmap.lazy_batch(|lazy| {
    ///     for b in &bitmaps_to_or {
    ///         *lazy |= b;
    ///     }
    ///     for b in &bitmaps_to_xor {
    ///         *lazy ^= b;
    ///     }
    /// });
    /// let mut bitmap2 = Bitmap::of(&[99]);
    /// for b in &bitmaps_to_or {
    ///     bitmap2 |= b;
    /// }
    /// for b in &bitmaps_to_xor {
    ///     bitmap2 ^= b;
    /// }
    /// assert_eq!(bitmap, bitmap2);
    /// assert_eq!(bitmap.to_vec(), [2, 10, 30, 99, 100, 1000, 1001]);
    /// ```
    ///
    /// The result the passed closure is returned from `lazy_batch`
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::create();
    /// let bitmaps_to_or = [Bitmap::of(&[1, 2, 5, 10]), Bitmap::of(&[1, 30, 100])];
    /// let total_added = bitmap.lazy_batch(|lazy| {
    ///     let mut total = 0;
    ///     for b in &bitmaps_to_or {
    ///         lazy.or_inplace(b, true);
    ///         total += b.cardinality();
    ///     }
    ///     total
    /// });
    /// assert_eq!(total_added, 7);
    #[doc(alias = "roaring_bitmap_repair_after_lazy")]
    pub fn lazy_batch<F, O>(&mut self, f: F) -> O
    where
        F: FnOnce(&mut LazyBitmap<'_>) -> O,
    {
        let mut lazy_bitmap = LazyBitmap { bitmap: self };
        let result = f(&mut lazy_bitmap);
        unsafe {
            ffi::roaring_bitmap_repair_after_lazy(&mut self.bitmap);
        }
        result
    }
}
