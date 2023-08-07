use super::Bitset;
use std::ffi::c_void;
use std::{fmt, ops};

impl Default for Bitset {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Bitset {
    fn clone(&self) -> Self {
        unsafe { Bitset::take_heap(ffi::bitset_copy(&self.bitset)) }
    }
}

impl Extend<usize> for Bitset {
    fn extend<T: IntoIterator<Item = usize>>(&mut self, iter: T) {
        for value in iter {
            self.set(value);
        }
    }
}

impl FromIterator<usize> for Bitset {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        let mut bitset = Bitset::new();
        bitset.extend(iter);
        bitset
    }
}

impl ops::ShlAssign<usize> for Bitset {
    #[inline]
    #[doc(alias = "bitset_shift_left")]
    fn shl_assign(&mut self, shift: usize) {
        unsafe { ffi::bitset_shift_left(&mut self.bitset, shift) };
    }
}

impl ops::ShrAssign<usize> for Bitset {
    #[inline]
    #[doc(alias = "bitset_shift_right")]
    fn shr_assign(&mut self, shift: usize) {
        unsafe { ffi::bitset_shift_right(&mut self.bitset, shift) };
    }
}

impl ops::BitOrAssign<&Bitset> for Bitset {
    #[inline]
    #[doc(alias = "bitset_inplace_union")]
    fn bitor_assign(&mut self, rhs: &Bitset) {
        let result = unsafe { ffi::bitset_inplace_union(&mut self.bitset, &rhs.bitset) };
        assert!(result);
    }
}

impl ops::BitAndAssign<&Bitset> for Bitset {
    #[inline]
    #[doc(alias = "bitset_inplace_intersection")]
    fn bitand_assign(&mut self, rhs: &Bitset) {
        unsafe { ffi::bitset_inplace_intersection(&mut self.bitset, &rhs.bitset) };
    }
}

impl ops::SubAssign<&Bitset> for Bitset {
    #[inline]
    #[doc(alias = "bitset_inplace_difference")]
    fn sub_assign(&mut self, rhs: &Bitset) {
        unsafe { ffi::bitset_inplace_difference(&mut self.bitset, &rhs.bitset) };
    }
}

impl ops::BitXorAssign<&Bitset> for Bitset {
    #[inline]
    #[doc(alias = "bitset_inplace_symmetric_difference")]
    fn bitxor_assign(&mut self, rhs: &Bitset) {
        unsafe { ffi::bitset_inplace_symmetric_difference(&mut self.bitset, &rhs.bitset) };
    }
}

impl fmt::Debug for Bitset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl Drop for Bitset {
    fn drop(&mut self) {
        unsafe {
            ffi::roaring_free(self.bitset.array.cast::<c_void>());
        }
    }
}
