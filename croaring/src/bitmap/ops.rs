use crate::BitmapView;
use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};

use super::Bitmap;

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

impl Default for Bitmap {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new bitmap from a slice of u32 values
///
/// # Examples
///
/// ```
/// use croaring::Bitmap;
///
/// let data: &[u32] = &[1, 2, 3];
///
/// let bitmap1 = Bitmap::from(data);
/// let bitmap2 = Bitmap::from_range(1..=3);
/// assert_eq!(bitmap1, bitmap2);
/// ```
impl From<&'_ [u32]> for Bitmap {
    #[inline]
    #[doc(alias = "roaring_bitmap_of_ptr")]
    fn from(values: &'_ [u32]) -> Self {
        Self::of(values)
    }
}

/// Create a new bitmap from an array of u32 values
///
/// # Examples
///
/// ```
/// use croaring::Bitmap;
///
/// let bitmap1 = Bitmap::from([1, 2, 3]);
/// let bitmap2 = Bitmap::from_range(1..=3);
/// assert_eq!(bitmap1, bitmap2);
/// ```
impl<const N: usize> From<[u32; N]> for Bitmap {
    #[inline]
    #[doc(alias = "roaring_bitmap_of_ptr")]
    fn from(values: [u32; N]) -> Self {
        Self::of(&values)
    }
}

impl PartialEq for Bitmap {
    #[inline]
    #[doc(alias = "roaring_bitmap_equals")]
    fn eq(&self, other: &Bitmap) -> bool {
        unsafe { ffi::roaring_bitmap_equals(&self.bitmap, &other.bitmap) }
    }
}

impl PartialEq<BitmapView<'_>> for Bitmap {
    #[inline]
    fn eq(&self, other: &BitmapView) -> bool {
        unsafe { ffi::roaring_bitmap_equals(&self.bitmap, &other.bitmap) }
    }
}

impl PartialEq<Bitmap> for BitmapView<'_> {
    #[inline]
    fn eq(&self, other: &Bitmap) -> bool {
        unsafe { ffi::roaring_bitmap_equals(&self.bitmap, &other.bitmap) }
    }
}

impl Eq for Bitmap {}

impl Clone for Bitmap {
    /// Create a copy of a Bitmap
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap1 = Bitmap::new();
    /// bitmap1.add(11);
    ///
    /// let bitmap2 = bitmap1.clone();
    ///
    /// assert_eq!(bitmap1, bitmap2);
    /// ```
    #[inline]
    fn clone(&self) -> Self {
        let mut result = Self::new();
        result.clone_from(self);
        result
    }

    #[doc(alias = "roaring_bitmap_overwrite")]
    fn clone_from(&mut self, source: &Self) {
        unsafe {
            let success = ffi::roaring_bitmap_overwrite(&mut self.bitmap, &source.bitmap);
            assert!(success, "Memory allocation failure cloning roaring bitmap");
        }
    }
}

impl Drop for Bitmap {
    #[allow(clippy::assertions_on_constants)]
    #[doc(alias = "roaring_bitmap_clear")]
    fn drop(&mut self) {
        // This depends somewhat heavily on the implementation of croaring,
        // Ensure this is still valid every time we update the version of croaring.
        const _: () = assert!(
            ffi::ROARING_VERSION_MAJOR == 1
                && ffi::ROARING_VERSION_MINOR == 3
                && ffi::ROARING_VERSION_REVISION == 0
        );

        // Per https://github.com/RoaringBitmap/CRoaring/blob/4f8dbdb0cc884626b20ef0cc9e891f701fe157cf/cpp/roaring.hh#L182
        // > By contract, calling roaring_bitmap_clear() is enough to
        // > release all auxiliary memory used by the structure.
        //
        // We do not currently expose a way to get a frozen bitmap, but if we ever do,
        // look at the roaring.hh destructor for implementation
        unsafe { ffi::roaring_bitmap_clear(&mut self.bitmap) }
    }
}

macro_rules! impl_binop {
    (
        impl $trait_name:ident {
            $(type $type_name:ident = $type_value:ty;)*

            $(#[$($attr:tt)*])*
            fn $fn_name:ident -> $ret_ty:ty as $alias:ident
        }
    ) => {
        impl_binop!{
            impl $trait_name {
                $(type $type_name = $type_value;)*

                $(#[$($attr)*])*
                fn $fn_name(self, other) -> $ret_ty {
                    self.$alias(&other)
                }
            }
        }
    };
    (
        impl $trait_name:ident {
            $(type $type_name:ident = $type_value:ty;)*

            $(#[$($attr:tt)*])*
            fn $fn_name:ident($self_ident:ident, $other_ident:ident) -> $ret_ty:ty
            $body:block
        }
    ) => {
        impl $trait_name for Bitmap {
            $(type $type_name = $type_value;)*

            $(#[$($attr)*])*
            fn $fn_name($self_ident, $other_ident: Bitmap) -> $ret_ty
            $body
        }

        impl $trait_name<&Bitmap> for Bitmap {
            $(type $type_name = $type_value;)*

            $(#[$($attr)*])*
            fn $fn_name($self_ident, $other_ident: &Bitmap) -> $ret_ty
            $body
        }

        impl $trait_name<Bitmap> for &Bitmap {
            $(type $type_name = $type_value;)*

            $(#[$($attr)*])*
            fn $fn_name($self_ident, $other_ident: Bitmap) -> $ret_ty
            $body
        }

        impl $trait_name<&Bitmap> for &Bitmap {
            $(type $type_name = $type_value;)*

            $(#[$($attr)*])*
            fn $fn_name($self_ident, $other_ident: &Bitmap) -> $ret_ty
            $body
        }

        impl $trait_name for BitmapView<'_> {
            $(type $type_name = $type_value;)*

            $(#[$($attr)*])*
            fn $fn_name($self_ident, $other_ident: BitmapView<'_>) -> $ret_ty
            $body
        }

        impl $trait_name<&BitmapView<'_>> for BitmapView<'_> {
            $(type $type_name = $type_value;)*

            $(#[$($attr)*])*
            fn $fn_name($self_ident, $other_ident: &BitmapView<'_>) -> $ret_ty
            $body
        }

        impl $trait_name<BitmapView<'_>> for &BitmapView<'_> {
            $(type $type_name = $type_value;)*

            $(#[$($attr)*])*
            fn $fn_name($self_ident, $other_ident: BitmapView<'_>) -> $ret_ty
            $body
        }

        impl $trait_name<&BitmapView<'_>> for &BitmapView<'_> {
            $(type $type_name = $type_value;)*

            $(#[$($attr)*])*
            fn $fn_name($self_ident, $other_ident: &BitmapView<'_>) -> $ret_ty
            $body
        }

        impl $trait_name<Bitmap> for BitmapView<'_> {
            $(type $type_name = $type_value;)*

            $(#[$($attr)*])*
            fn $fn_name($self_ident, $other_ident: Bitmap) -> $ret_ty
            $body
        }

        impl $trait_name<&Bitmap> for BitmapView<'_> {
            $(type $type_name = $type_value;)*

            $(#[$($attr)*])*
            fn $fn_name($self_ident, $other_ident: &Bitmap) -> $ret_ty
            $body
        }

        impl $trait_name<&Bitmap> for &BitmapView<'_> {
            $(type $type_name = $type_value;)*

            $(#[$($attr)*])*
            fn $fn_name($self_ident, $other_ident: &Bitmap) -> $ret_ty
            $body
        }

        impl $trait_name<Bitmap> for &BitmapView<'_> {
            $(type $type_name = $type_value;)*

            $(#[$($attr)*])*
            fn $fn_name($self_ident, $other_ident: Bitmap) -> $ret_ty
            $body
        }
    };
}

macro_rules! impl_binop_assign {
    (
        impl $trait_name:ident {
            $(#[$($attr:tt)*])*
            fn $fn_name:ident as $alias:ident
        }
    ) => {
        impl $trait_name for Bitmap {
            $(#[$($attr)*])*
            fn $fn_name(&mut self, other: Bitmap) {
                self.$alias(&other)
            }
        }

        impl $trait_name<&'_ Bitmap> for Bitmap {
            $(#[$($attr)*])*
            fn $fn_name(&mut self, other: &Bitmap) {
                self.$alias(other)
            }
        }

        impl $trait_name<BitmapView<'_>> for Bitmap {
            $(#[$($attr)*])*
            fn $fn_name(&mut self, other: BitmapView<'_>) {
                self.$alias(&other)
            }
        }

        impl $trait_name<&BitmapView<'_>> for Bitmap {
            $(#[$($attr)*])*
            fn $fn_name(&mut self, other: &BitmapView<'_>) {
                self.$alias(other)
            }
        }
    };
}

impl_binop! {
    impl BitAnd {
        type Output = Bitmap;

        /// Syntactic sugar for `.and`
        ///
        /// # Examples
        ///
        /// ```
        /// use croaring::Bitmap;
        ///
        /// let mut bitmap1 = Bitmap::new();
        /// bitmap1.add(1);
        ///
        /// let mut bitmap2 = Bitmap::new();
        /// bitmap2.add(1);
        /// bitmap2.add(2);
        ///
        /// let bitmap3 = bitmap1 & bitmap2;
        ///
        /// assert!(bitmap3.contains(1));
        /// assert!(!bitmap3.contains(2));
        /// ```
        #[inline]
        #[doc(alias = "roaring_bitmap_and")]
        fn bitand -> Bitmap as and
    }
}

impl_binop! {
    impl BitOr {
        type Output = Bitmap;

        /// Syntatic sugar for `.or`
        ///
        /// # Examples
        ///
        /// ```
        /// use croaring::Bitmap;
        ///
        /// let bitmap1 = Bitmap::of(&[15]);
        /// let bitmap2 = Bitmap::of(&[25]);
        ///
        /// let bitmap3 = bitmap1 | bitmap2;
        ///
        /// assert!(bitmap3.cardinality() == 2);
        /// assert!(bitmap3.contains(15));
        /// assert!(bitmap3.contains(25));
        /// ```
        #[inline]
        #[doc(alias = "roaring_bitmap_or")]
        fn bitor -> Bitmap as or
    }
}

impl_binop! {
    impl BitXor {
        type Output = Bitmap;

        /// Syntatic sugar for `.xor`
        ///
        /// # Examples
        ///
        /// ```
        /// use croaring::Bitmap;
        ///
        /// let bitmap1 = Bitmap::of(&[15, 25]);
        /// let bitmap2 = Bitmap::of(&[25, 35]);
        ///
        /// let bitmap3 = bitmap1 ^ bitmap2;
        ///
        /// assert!(bitmap3.cardinality() == 2);
        /// assert!(bitmap3.contains(15));
        /// assert!(!bitmap3.contains(25));
        /// assert!(bitmap3.contains(35));
        /// ```
        #[inline]
        #[doc(alias = "roaring_bitmap_xor")]
        fn bitxor -> Bitmap as xor
    }
}

impl_binop! {
    impl Sub {
        type Output = Bitmap;

        /// Syntatic sugar for `.andnot`
        ///
        /// # Examples
        ///
        /// ```
        /// use croaring::Bitmap;
        ///
        /// let bitmap1 = Bitmap::of(&[15, 25]);
        /// let bitmap2 = Bitmap::of(&[25, 35]);
        ///
        /// let bitmap3 = bitmap1 - bitmap2;
        ///
        /// assert_eq!(bitmap3.cardinality(), 1);
        /// assert!(bitmap3.contains(15));
        /// assert!(!bitmap3.contains(25));
        /// assert!(!bitmap3.contains(35));
        /// ```
        #[inline]
        #[doc(alias = "andnot")]
        #[doc(alias = "roaring_bitmap_andnot")]
        fn sub -> Bitmap as andnot
    }
}

impl_binop_assign! {
    impl BitAndAssign {
        /// Syntactic sugar for `.and_inplace`
        ///
        /// # Examples
        ///
        /// ```
        /// use croaring::Bitmap;
        ///
        /// let mut bitmap1 = Bitmap::of(&[15]);
        /// let bitmap2 = Bitmap::of(&[25]);
        /// let mut bitmap3 = Bitmap::of(&[15]);
        /// let bitmap4 = Bitmap::of(&[15, 25]);
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
        #[doc(alias = "roaring_bitmap_and_inplace")]
        fn bitand_assign as and_inplace
    }
}

impl_binop_assign! {
    impl BitOrAssign {
        /// Syntatic sugar for `.or_inplace`
        ///
        /// # Examples
        ///
        /// ```
        /// use croaring::Bitmap;
        ///
        /// let mut bitmap1 = Bitmap::of(&[15]);
        /// let bitmap2 = Bitmap::of(&[25]);
        ///
        /// bitmap1 |= bitmap2;
        ///
        /// assert!(bitmap1.cardinality() == 2);
        /// assert!(bitmap1.contains(15));
        /// assert!(bitmap1.contains(25));
        /// ```
        #[inline]
        #[doc(alias = "roaring_bitmap_or_inplace")]
        fn bitor_assign as or_inplace
    }
}

impl_binop_assign! {
    impl BitXorAssign {
        /// Syntatic sugar for `.xor_inplace`
        ///
        /// # Examples
        ///
        /// ```
        /// use croaring::Bitmap;
        ///
        /// let mut bitmap1 = Bitmap::of(&[15, 25]);
        /// let bitmap2 = Bitmap::of(&[25, 35]);
        ///
        /// bitmap1 ^= bitmap2;
        ///
        /// assert!(bitmap1.cardinality() == 2);
        /// assert!(bitmap1.contains(15));
        /// assert!(!bitmap1.contains(25));
        /// assert!(bitmap1.contains(35));
        /// ```
        #[inline]
        #[doc(alias = "roaring_bitmap_xor_inplace")]
        fn bitxor_assign as xor_inplace
    }
}

impl_binop_assign! {
    impl SubAssign {
        /// Syntatic sugar for `.andnot_inplace`
        ///
        /// # Examples
        ///
        /// ```
        /// use croaring::Bitmap;
        ///
        /// let mut bitmap1 = Bitmap::of(&[15, 25]);
        /// let bitmap2 = Bitmap::of(&[25, 35]);
        ///
        /// bitmap1 -= bitmap2;
        ///
        /// assert_eq!(bitmap1.cardinality(), 1);
        /// assert!(bitmap1.contains(15));
        /// assert!(!bitmap1.contains(25));
        /// assert!(!bitmap1.contains(35));
        /// ```
        #[inline]
        #[doc(alias = "andnot_inplace")]
        #[doc(alias = "roaring_bitmap_andnot_inplace")]
        fn sub_assign as andnot_inplace
    }
}
