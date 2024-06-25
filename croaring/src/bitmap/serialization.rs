use super::{Bitmap, BitmapView};
use crate::serialization::{Frozen, Native, Portable};

use core::ffi::{c_char, c_void};

/// Trait for different formats of bitmap serialization
pub trait Serializer: crate::sealed::Sealed {
    /// The required alignment for the serialized data
    #[doc(hidden)]
    const REQUIRED_ALIGNMENT: usize = 1;

    /// Serialize a bitmap into bytes, using the provided vec buffer to store the serialized data
    ///
    /// Note that some serializers ([Frozen]) may require that the bitmap is aligned specially,
    /// this method will ensure that the returned slice of bytes is aligned correctly, by adding
    /// additional padding before the serialized data if required.
    ///
    /// The contents of the provided vec buffer will not be overwritten: only new data will be
    /// appended to the end of the buffer. If the buffer has enough capacity, and the current
    /// end of the buffer is correctly aligned, then no additional allocations will be performed.
    #[doc(hidden)]
    #[cfg(feature = "alloc")]
    fn serialize_into_vec<'a>(bitmap: &Bitmap, dst: &'a mut alloc::vec::Vec<u8>) -> &'a mut [u8] {
        let len = Self::get_serialized_size_in_bytes(bitmap);
        let spare_capacity =
            crate::serialization::get_aligned_spare_capacity(dst, Self::REQUIRED_ALIGNMENT, len);
        let data_start;
        unsafe {
            Self::raw_serialize(bitmap, spare_capacity.as_mut_ptr().cast::<c_char>());
            data_start = dst.len();
            let total_len = data_start.checked_add(len).unwrap();
            dst.set_len(total_len);
        }

        &mut dst[data_start..]
    }

    #[doc(hidden)]
    fn try_serialize_into_aligned<'a>(bitmap: &Bitmap, dst: &'a mut [u8]) -> Option<&'a mut [u8]> {
        let offset = dst.as_ptr().align_offset(Self::REQUIRED_ALIGNMENT);
        let offset_dst = dst.get_mut(offset..)?;
        let len = Self::try_serialize_into(bitmap, offset_dst)?;
        Some(&mut dst[offset..offset + len])
    }

    /// Serialize a bitmap into bytes, using the provided buffer to store the serialized data
    ///
    /// This method does not require the buffer to be aligned, and will return `None` if the buffer
    /// is not large enough to store the serialized data.
    ///
    /// This is a niche method, and is not recommended for general use. The
    /// [`Bitmap::serialize_into_vec`]/[`Bitmap::try_serialize_into`] methods should usually be used
    /// instead of this method.
    fn try_serialize_into(bitmap: &Bitmap, dst: &mut [u8]) -> Option<usize> {
        let required_len = Self::get_serialized_size_in_bytes(bitmap);
        if dst.len() < required_len {
            return None;
        }
        unsafe {
            Self::raw_serialize(bitmap, dst.as_mut_ptr().cast::<c_char>());
        }
        Some(required_len)
    }

    /// Get the number of bytes required to serialize this bitmap
    ///
    /// This does not include any additional padding which may be required to align the bitmap
    #[doc(hidden)]
    fn get_serialized_size_in_bytes(bitmap: &Bitmap) -> usize;

    #[doc(hidden)]
    unsafe fn raw_serialize(bitmap: &Bitmap, dst: *mut c_char);
}

/// Trait for different formats of bitmap deserialization
pub trait Deserializer: crate::sealed::Sealed {
    /// Try to deserialize a bitmap from the beginning of the provided buffer
    ///
    /// The [`Bitmap::try_deserialize`] method should usually be used instead of this method
    /// directly.
    ///
    /// If the buffer starts with the serialized representation of a bitmap, then
    /// this method will return a new bitmap containing the deserialized data.
    ///
    /// If the buffer does not start with a serialized bitmap (or contains an invalidly
    /// truncated bitmap), then this method will return `None`.
    ///
    /// To determine how many bytes were consumed from the buffer, use the
    /// [`Serializer::get_serialized_size_in_bytes`] method on the returned bitmap.
    #[doc(hidden)]
    fn try_deserialize(buffer: &[u8]) -> Option<Bitmap>;

    /// Deserialize a bitmap from the beginning of the provided buffer
    ///
    /// # Safety
    ///
    /// Unlike its safe counterpart ([`Self::try_deserialize`]) this function assumes the data is
    /// valid, passing data which does not contain/start with a bitmap serialized with this format
    /// will result in undefined behavior.
    #[doc(hidden)]
    unsafe fn try_deserialize_unchecked(buffer: &[u8]) -> Bitmap;
}

/// Trait for different formats of bitmap deserialization into a view without copying
pub trait ViewDeserializer: crate::sealed::Sealed {
    /// Create a bitmap view using the passed data
    ///
    /// # Safety
    /// * `data` must be the result of serializing a roaring bitmap in this format.
    /// * Its beginning must be aligned properly for this format.
    /// * data.len() must be equal exactly to the size of the serialized bitmap.
    ///
    /// See [`BitmapView::deserialize`] for examples.
    #[doc(hidden)]
    unsafe fn deserialize_view(data: &[u8]) -> BitmapView<'_>;
}

impl crate::sealed::Sealed for Portable {}
impl Serializer for Portable {
    /// Computes the serialized size in bytes of the Bitmap in portable format.
    /// See [`Bitmap::get_serialized_size_in_bytes`] for examples.
    #[doc(alias = "roaring_bitmap_portable_size_in_bytes")]
    fn get_serialized_size_in_bytes(bitmap: &Bitmap) -> usize {
        unsafe { ffi::roaring_bitmap_portable_size_in_bytes(&bitmap.bitmap) }
    }

    unsafe fn raw_serialize(bitmap: &Bitmap, dst: *mut c_char) {
        unsafe {
            ffi::roaring_bitmap_portable_serialize(&bitmap.bitmap, dst);
        }
    }
}

impl Deserializer for Portable {
    /// Given a serialized bitmap as slice of bytes in portable format, returns a `Bitmap` instance.
    /// See [`Bitmap::try_deserialize`] for examples.
    #[doc(alias = "roaring_bitmap_portable_deserialize_safe")]
    fn try_deserialize(buffer: &[u8]) -> Option<Bitmap> {
        unsafe {
            let bitmap = ffi::roaring_bitmap_portable_deserialize_safe(
                buffer.as_ptr().cast::<c_char>(),
                buffer.len(),
            );

            if bitmap.is_null() {
                return None;
            }

            let bitmap = Bitmap::take_heap(bitmap);
            if bitmap.internal_validate().is_ok() {
                Some(bitmap)
            } else {
                None
            }
        }
    }

    #[doc(alias = "roaring_bitmap_portable_deserialize")]
    unsafe fn try_deserialize_unchecked(buffer: &[u8]) -> Bitmap {
        let bitmap = ffi::roaring_bitmap_portable_deserialize(buffer.as_ptr().cast::<c_char>());
        Bitmap::take_heap(bitmap)
    }
}

impl ViewDeserializer for Portable {
    /// Read bitmap from a serialized buffer
    ///
    /// This is meant to be compatible with the Java and Go versions
    ///
    /// # Safety
    /// * `data` must be the result of serializing a roaring bitmap in portable mode
    ///   (following `https://github.com/RoaringBitmap/RoaringFormatSpec`), for example, with
    ///   [`Bitmap::serialize`]
    /// * Using this function (or the returned bitmap in any way) may execute unaligned memory accesses
    ///
    #[doc(alias = "roaring_bitmap_portable_deserialize_frozen")]
    unsafe fn deserialize_view(data: &[u8]) -> BitmapView<'_> {
        // portable_deserialize_size does some amount of checks, and returns zero if data cannot be valid
        debug_assert_ne!(
            ffi::roaring_bitmap_portable_deserialize_size(data.as_ptr().cast(), data.len()),
            0,
        );
        let roaring = ffi::roaring_bitmap_portable_deserialize_frozen(data.as_ptr().cast());
        BitmapView::take_heap(roaring)
    }
}

impl crate::sealed::Sealed for Native {}
impl Serializer for Native {
    /// Computes the serialized size in bytes of the Bitmap in native format.
    /// See [`Bitmap::get_serialized_size_in_bytes`] for examples.
    #[doc(alias = "roaring_bitmap_size_in_bytes")]
    fn get_serialized_size_in_bytes(bitmap: &Bitmap) -> usize {
        unsafe { ffi::roaring_bitmap_size_in_bytes(&bitmap.bitmap) }
    }

    unsafe fn raw_serialize(bitmap: &Bitmap, dst: *mut c_char) {
        unsafe {
            ffi::roaring_bitmap_serialize(&bitmap.bitmap, dst);
        }
    }
}

impl Deserializer for Native {
    /// Given a serialized bitmap as slice of bytes in native format, returns a `Bitmap` instance.
    /// See [`Bitmap::try_deserialize`] for examples.
    #[doc(alias = "roaring_bitmap_deserialize_safe")]
    fn try_deserialize(buffer: &[u8]) -> Option<Bitmap> {
        unsafe {
            let bitmap = ffi::roaring_bitmap_deserialize_safe(
                buffer.as_ptr().cast::<c_void>(),
                buffer.len(),
            );

            if bitmap.is_null() {
                return None;
            }
            let bitmap = Bitmap::take_heap(bitmap);
            if bitmap.internal_validate().is_ok() {
                Some(bitmap)
            } else {
                None
            }
        }
    }

    #[doc(alias = "roaring_bitmap_deserialize")]
    unsafe fn try_deserialize_unchecked(buffer: &[u8]) -> Bitmap {
        let bitmap = ffi::roaring_bitmap_deserialize(buffer.as_ptr().cast::<c_void>());
        Bitmap::take_heap(bitmap)
    }
}

impl crate::sealed::Sealed for Frozen {}
impl Serializer for Frozen {
    // Defer to the innate const on Frozen
    const REQUIRED_ALIGNMENT: usize = Self::REQUIRED_ALIGNMENT;

    /// Computes the serialized size in bytes of the Bitmap in frozen format.
    /// See [`Bitmap::get_serialized_size_in_bytes`] for examples.
    #[doc(alias = "roaring_bitmap_frozen_size_in_bytes")]
    fn get_serialized_size_in_bytes(bitmap: &Bitmap) -> usize {
        unsafe { ffi::roaring_bitmap_frozen_size_in_bytes(&bitmap.bitmap) }
    }

    unsafe fn raw_serialize(bitmap: &Bitmap, dst: *mut c_char) {
        unsafe {
            ffi::roaring_bitmap_frozen_serialize(&bitmap.bitmap, dst);
        }
    }
}

impl ViewDeserializer for Frozen {
    /// Create a frozen bitmap view using the passed data
    ///
    /// # Safety
    /// * `data` must be the result of serializing a roaring bitmap in frozen mode
    ///   (in c with `roaring_bitmap_frozen_serialize`, or via [`Bitmap::try_serialize_into::<Frozen>`]).
    /// * Its beginning must be aligned by 32 bytes.
    /// * data.len() must be equal exactly to the size of the frozen bitmap.
    ///
    /// See [`BitmapView::deserialize`] for examples.
    unsafe fn deserialize_view(data: &[u8]) -> BitmapView<'_> {
        assert_eq!(data.as_ptr() as usize % Self::REQUIRED_ALIGNMENT, 0);

        let roaring = ffi::roaring_bitmap_frozen_view(data.as_ptr().cast(), data.len());
        BitmapView::take_heap(roaring)
    }
}
