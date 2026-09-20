use crate::bitmap64::serialization::ViewDeserializer;
use crate::bitmap64::{Bitmap64, Bitmap64View};
use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;

impl<'a> Bitmap64View<'a> {
    /// Creates a bitmap view of a slice of data without copying.
    ///
    /// This function returns an option, which will return `None` if the data is not a valid bitmap,
    /// however, this is only done on a best-effort basis, and may not catch all invalid data.
    /// This function is _only_ safe to call if the caller _knows_ that the data is a valid bitmap.
    ///
    /// [`Portable`][crate::Portable] data does not require alignment. [`Frozen`][crate::Frozen]
    /// data must be aligned to 64 bytes. The returned view borrows `data`, which must not be
    /// modified for the lifetime of the view.
    ///
    /// Portable views are unavailable on big-endian systems, where this function returns `None`.
    /// Use [`Bitmap64::try_deserialize`] to create an owned bitmap on those systems.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::{Bitmap64, Bitmap64View, Portable};
    /// let orig_bitmap = Bitmap64::of(&[1, 2, 3, 4]);
    /// let mut buf = [0; 1024];
    /// let data: &[u8] = orig_bitmap.try_serialize_into::<Portable>(&mut buf).unwrap();
    /// let view = unsafe { Bitmap64View::deserialize::<Portable>(data) }.unwrap();
    /// assert!(view.contains_range(1..=4));
    /// assert_eq!(orig_bitmap, view);
    /// ```
    ///
    /// # Safety
    ///
    /// The data must be the result of serializing a bitmap with the same serialization format.
    /// It must remain unchanged for the lifetime of the returned view and meet the alignment
    /// requirements of the selected format.
    #[must_use]
    pub unsafe fn deserialize<S: ViewDeserializer>(data: &'a [u8]) -> Option<Self> {
        unsafe {
            let bitmap_ptr = S::deserialize_view(data);
            if bitmap_ptr.is_null() {
                return None;
            }
            Some(Self {
                bitmap: Bitmap64::take_heap(bitmap_ptr),
                phantom: PhantomData,
            })
        }
    }

    /// Create an owned, mutable bitmap from this view
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::{Bitmap64, Bitmap64View, Frozen};
    ///
    /// let mut orig_bitmap = Bitmap64::of(&[1, 2, 3, 4]);
    /// orig_bitmap.shrink_to_fit();
    /// let mut buf = [0; 1024];
    /// let data: &[u8] = orig_bitmap.try_serialize_into::<Frozen>(&mut buf).unwrap();
    /// let view: Bitmap64View = unsafe { Bitmap64View::deserialize::<Frozen>(data) }.unwrap();
    /// # assert_eq!(view, orig_bitmap);
    /// let mut mutable_bitmap: Bitmap64 = view.to_bitmap64();
    /// assert_eq!(view, mutable_bitmap);
    /// mutable_bitmap.add(10);
    /// assert!(!view.contains(10));
    /// assert!(mutable_bitmap.contains(10));
    /// ```
    #[must_use]
    pub fn to_bitmap64(&self) -> Bitmap64 {
        self.bitmap.clone()
    }
}

impl<'a> Deref for Bitmap64View<'a> {
    type Target = Bitmap64;

    fn deref(&self) -> &Self::Target {
        &self.bitmap
    }
}

impl<'a, 'b> PartialEq<Bitmap64View<'a>> for Bitmap64View<'b> {
    fn eq(&self, other: &Bitmap64View<'a>) -> bool {
        self.bitmap == other.bitmap
    }
}

impl Eq for Bitmap64View<'_> {}

impl PartialEq<Bitmap64View<'_>> for Bitmap64 {
    fn eq(&self, other: &Bitmap64View<'_>) -> bool {
        *self == other.bitmap
    }
}

impl PartialEq<Bitmap64> for Bitmap64View<'_> {
    fn eq(&self, other: &Bitmap64) -> bool {
        self.bitmap == *other
    }
}

impl fmt::Debug for Bitmap64View<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.bitmap.fmt(f)
    }
}
