use super::serialization::ViewDeserializer;
use super::{Bitmap, BitmapView};
use ffi::roaring_bitmap_t;
use std::marker::PhantomData;
use std::ops::Deref;
use std::{fmt, mem};

#[inline]
const fn original_bitmap_ptr(bitmap: &roaring_bitmap_t) -> *const roaring_bitmap_t {
    // The implementation must put the containers array immediately after the bitmap pointer
    bitmap
        .high_low_container
        .containers
        .cast::<roaring_bitmap_t>()
        // wrapping_sub to ensure we can still check this ptr against the original even if
        // this isn't actually the correct pointer because of a change in CRoaring implementation
        .wrapping_sub(1)
}

impl<'a> BitmapView<'a> {
    #[inline]
    #[allow(clippy::assertions_on_constants)]
    pub(crate) unsafe fn take_heap(p: *const roaring_bitmap_t) -> Self {
        // This depends somewhat heavily on the implementation of croaring,
        // In particular, that `roaring_bitmap_t` doesn't store any pointers into itself
        // (it can be moved safely), and a "frozen" bitmap is stored in an arena, and the
        // `containers` array is stored immediately after the roaring_bitmap_t data.
        // Ensure this is still valid every time we update
        // the version of croaring.
        const _: () = assert!(ffi::ROARING_VERSION_MAJOR == 2 && ffi::ROARING_VERSION_MINOR == 0);

        assert!(!p.is_null());

        // We will use this in the Drop implementation to re-create this pointer to pass to roaring_bitmap_free
        // If this fails, we would pass junk to roaring_bitmap_free in Drop.
        assert_eq!(p, original_bitmap_ptr(&*p));

        Self {
            bitmap: *p,
            phantom: PhantomData,
        }
    }

    /// Create a bitmap view of a slice of data without copying
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::{Bitmap, BitmapView, Portable};
    /// let orig_bitmap = Bitmap::of(&[1, 2, 3, 4]);
    /// let data: Vec<u8> = orig_bitmap.serialize::<Portable>();
    /// let view = unsafe { BitmapView::deserialize::<Portable>(&data) };
    /// assert!(view.contains_range(1..=4));
    /// assert_eq!(orig_bitmap, view);
    /// ```
    ///
    /// # Safety
    ///
    /// The data must be the result of serializing a bitmap with the same serialization format
    #[must_use]
    pub unsafe fn deserialize<S: ViewDeserializer>(data: &'a [u8]) -> Self {
        S::deserialize_view(data)
    }

    /// Create an owned, mutable bitmap from this view
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::{Bitmap, BitmapView, Portable};
    ///
    /// let orig_bitmap = Bitmap::of(&[1, 2, 3, 4]);
    /// let data = orig_bitmap.serialize::<Portable>();
    /// let view: BitmapView = unsafe { BitmapView::deserialize::<Portable>(&data) };
    /// # assert_eq!(view, orig_bitmap);
    /// let mut mutable_bitmap: Bitmap = view.to_bitmap();
    /// assert_eq!(view, mutable_bitmap);
    /// mutable_bitmap.add(10);
    /// assert!(!view.contains(10));
    /// assert!(mutable_bitmap.contains(10));
    /// ```
    #[must_use]
    pub fn to_bitmap(&self) -> Bitmap {
        (**self).clone()
    }
}

impl<'a> Deref for BitmapView<'a> {
    type Target = Bitmap;

    fn deref(&self) -> &Self::Target {
        const _: () = assert!(mem::size_of::<Bitmap>() == mem::size_of::<BitmapView>());
        const _: () = assert!(mem::align_of::<Bitmap>() == mem::align_of::<BitmapView>());
        // SAFETY:
        //   Bitmap and BitmapView are repr(transparent), and both only wrap a roaring_bitmap_t
        //   Bitmap provides no features with a shared reference which modifies the underlying bitmap
        unsafe { mem::transmute::<&BitmapView, &Bitmap>(self) }
    }
}

impl PartialEq for BitmapView<'_> {
    fn eq(&self, other: &Self) -> bool {
        Bitmap::eq(self, other)
    }
}

impl Eq for BitmapView<'_> {}

impl<'a> Drop for BitmapView<'a> {
    fn drop(&mut self) {
        // Based heavily on the c++ wrapper included in CRoaring
        //
        // The roaring member variable copies the `roaring_bitmap_t` and
        // nested `roaring_array_t` structures by value and is freed in the
        // constructor, however the underlying memory arena used for the
        // container data is not freed with it. Here we derive the arena
        // pointer from the second arena allocation in
        // `roaring_bitmap_frozen_view` and free it as well.
        unsafe {
            ffi::roaring_bitmap_free(original_bitmap_ptr(&self.bitmap));
        }
    }
}

impl<'a> fmt::Debug for BitmapView<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}
