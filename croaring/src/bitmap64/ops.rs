use crate::Bitmap64;
use core::fmt;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};
use ffi::roaring64_bitmap_copy;

impl fmt::Debug for Bitmap64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.cardinality() < 32 {
            write!(f, "Bitmap64<[")?;
            let mut first = true;
            for value in self.iter() {
                let prefix = if first {
                    first = false;
                    ""
                } else {
                    ", "
                };
                write!(f, "{prefix}{value}")?;
            }
            write!(f, "]>")?;
            Ok(())
        } else {
            write!(
                f,
                "Bitmap64<{:?} values between {:?} and {:?}>",
                self.cardinality(),
                self.minimum().unwrap(),
                self.maximum().unwrap()
            )
        }
    }
}

impl Default for Bitmap64 {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&'_ [u64]> for Bitmap64 {
    #[inline]
    #[doc(alias = "roaring64_bitmap_of_ptr")]
    fn from(slice: &[u64]) -> Self {
        Self::of(slice)
    }
}

impl<const N: usize> From<[u64; N]> for Bitmap64 {
    #[inline]
    #[doc(alias = "roaring64_bitmap_of_ptr")]
    fn from(slice: [u64; N]) -> Self {
        Self::of(&slice)
    }
}

impl PartialEq for Bitmap64 {
    #[inline]
    #[doc(alias = "roaring64_bitmap_equals")]
    fn eq(&self, other: &Self) -> bool {
        unsafe { ffi::roaring64_bitmap_equals(self.raw.as_ptr(), other.raw.as_ptr()) }
    }
}

impl Eq for Bitmap64 {}

impl Clone for Bitmap64 {
    #[inline]
    #[doc(alias = "roaring64_bitmap_copy")]
    fn clone(&self) -> Self {
        unsafe {
            let raw = roaring64_bitmap_copy(self.raw.as_ptr());
            Self::take_heap(raw)
        }
    }
}

impl Drop for Bitmap64 {
    fn drop(&mut self) {
        unsafe {
            ffi::roaring64_bitmap_free(self.raw.as_ptr());
        }
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
        impl $trait_name for Bitmap64 {
            $(type $type_name = $type_value;)*

            $(#[$($attr)*])*
            fn $fn_name($self_ident, $other_ident: Bitmap64) -> $ret_ty
            $body
        }

        impl $trait_name<&Bitmap64> for Bitmap64 {
            $(type $type_name = $type_value;)*

            $(#[$($attr)*])*
            fn $fn_name($self_ident, $other_ident: &Bitmap64) -> $ret_ty
            $body
        }

        impl $trait_name<Bitmap64> for &Bitmap64 {
            $(type $type_name = $type_value;)*

            $(#[$($attr)*])*
            fn $fn_name($self_ident, $other_ident: Bitmap64) -> $ret_ty
            $body
        }

        impl $trait_name<&Bitmap64> for &Bitmap64 {
            $(type $type_name = $type_value;)*

            $(#[$($attr)*])*
            fn $fn_name($self_ident, $other_ident: &Bitmap64) -> $ret_ty
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
        impl $trait_name for Bitmap64 {
            $(#[$($attr)*])*
            fn $fn_name(&mut self, other: Bitmap64) {
                self.$alias(&other)
            }
        }

        impl $trait_name<&'_ Bitmap64> for Bitmap64 {
            $(#[$($attr)*])*
            fn $fn_name(&mut self, other: &Bitmap64) {
                self.$alias(other)
            }
        }
    };
}

impl_binop! {
    impl BitAnd {
        type Output = Bitmap64;

        /// Syntactic sugar for `.and`
        ///
        /// # Examples
        ///
        /// ```
        /// use croaring::Bitmap64;
        ///
        /// let mut bitmap1 = Bitmap64::new();
        /// bitmap1.add(1);
        ///
        /// let mut bitmap2 = Bitmap64::new();
        /// bitmap2.add(1);
        /// bitmap2.add(2);
        ///
        /// let bitmap3 = bitmap1 & bitmap2;
        ///
        /// assert!(bitmap3.contains(1));
        /// assert!(!bitmap3.contains(2));
        /// ```
        #[inline]
        #[doc(alias = "roaring64_bitmap_and")]
        fn bitand -> Bitmap64 as and
    }
}

impl_binop! {
    impl BitOr {
        type Output = Bitmap64;

        /// Syntatic sugar for `.or`
        ///
        /// # Examples
        ///
        /// ```
        /// use croaring::Bitmap64;
        ///
        /// let bitmap1 = Bitmap64::of(&[15]);
        /// let bitmap2 = Bitmap64::of(&[25]);
        ///
        /// let bitmap3 = bitmap1 | bitmap2;
        ///
        /// assert!(bitmap3.cardinality() == 2);
        /// assert!(bitmap3.contains(15));
        /// assert!(bitmap3.contains(25));
        /// ```
        #[inline]
        #[doc(alias = "roaring64_bitmap_or")]
        fn bitor -> Bitmap64 as or
    }
}

impl_binop! {
    impl BitXor {
        type Output = Bitmap64;

        /// Syntatic sugar for `.xor`
        ///
        /// # Examples
        ///
        /// ```
        /// use croaring::Bitmap64;
        ///
        /// let bitmap1 = Bitmap64::of(&[15, 25]);
        /// let bitmap2 = Bitmap64::of(&[25, 35]);
        ///
        /// let bitmap3 = bitmap1 ^ bitmap2;
        ///
        /// assert!(bitmap3.cardinality() == 2);
        /// assert!(bitmap3.contains(15));
        /// assert!(!bitmap3.contains(25));
        /// assert!(bitmap3.contains(35));
        /// ```
        #[inline]
        #[doc(alias = "roaring64_bitmap_xor")]
        fn bitxor -> Bitmap64 as xor
    }
}

impl_binop! {
    impl Sub {
        type Output = Bitmap64;

        /// Syntatic sugar for `.andnot`
        ///
        /// # Examples
        ///
        /// ```
        /// use croaring::Bitmap64;
        ///
        /// let bitmap1 = Bitmap64::of(&[15, 25]);
        /// let bitmap2 = Bitmap64::of(&[25, 35]);
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
        #[doc(alias = "roaring64_bitmap_andnot")]
        fn sub -> Bitmap64 as andnot
    }
}

impl_binop_assign! {
    impl BitAndAssign {
        /// Syntactic sugar for `.and_inplace`
        ///
        /// # Examples
        ///
        /// ```
        /// use croaring::Bitmap64;
        ///
        /// let mut bitmap1 = Bitmap64::of(&[15]);
        /// let bitmap2 = Bitmap64::of(&[25]);
        /// let mut bitmap3 = Bitmap64::of(&[15]);
        /// let bitmap4 = Bitmap64::of(&[15, 25]);
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
        #[doc(alias = "roaring64_bitmap_and_inplace")]
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
        /// use croaring::Bitmap64;
        ///
        /// let mut bitmap1 = Bitmap64::of(&[15]);
        /// let bitmap2 = Bitmap64::of(&[25]);
        ///
        /// bitmap1 |= bitmap2;
        ///
        /// assert!(bitmap1.cardinality() == 2);
        /// assert!(bitmap1.contains(15));
        /// assert!(bitmap1.contains(25));
        /// ```
        #[inline]
        #[doc(alias = "roaring64_bitmap_or_inplace")]
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
        /// use croaring::Bitmap64;
        ///
        /// let mut bitmap1 = Bitmap64::of(&[15, 25]);
        /// let bitmap2 = Bitmap64::of(&[25, 35]);
        ///
        /// bitmap1 ^= bitmap2;
        ///
        /// assert!(bitmap1.cardinality() == 2);
        /// assert!(bitmap1.contains(15));
        /// assert!(!bitmap1.contains(25));
        /// assert!(bitmap1.contains(35));
        /// ```
        #[inline]
        #[doc(alias = "roaring64_bitmap_xor_inplace")]
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
        /// use croaring::Bitmap64;
        ///
        /// let mut bitmap1 = Bitmap64::of(&[15, 25]);
        /// let bitmap2 = Bitmap64::of(&[25, 35]);
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
        #[doc(alias = "roaring64_bitmap_andnot_inplace")]
        fn sub_assign as andnot_inplace
    }
}
