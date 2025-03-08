use crate::{Bitmap64, Frozen, Portable};
use core::ffi::c_char;
use core::num::NonZeroUsize;

/// Trait for different formats of bitmap64 serialization
pub trait Serializer: crate::sealed::Sealed {
    /// The required alignment for the serialized data
    #[doc(hidden)]
    const REQUIRED_ALIGNMENT: usize = 1;

    /// Serialize a bitmap into bytes, using the provided vec buffer to store the serialized data
    ///
    /// Note that some serializers ([Frozen][crate::Frozen]) may require that the
    /// bitmap is aligned specially, this method will ensure that the returned slice of bytes is
    /// aligned correctly, by adding additional padding before the serialized data if required.
    ///
    /// The contents of the provided vec buffer will not be overwritten: only new data will be
    /// appended to the end of the buffer. If the buffer has enough capacity, and the current
    /// end of the buffer is correctly aligned, then no additional allocations will be performed.
    #[doc(hidden)]
    #[cfg(feature = "alloc")]
    fn serialize_into_vec<'a>(bitmap: &Bitmap64, dst: &'a mut alloc::vec::Vec<u8>) -> &'a mut [u8] {
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
    fn try_serialize_into_aligned<'a>(
        bitmap: &Bitmap64,
        dst: &'a mut [u8],
    ) -> Option<&'a mut [u8]> {
        let offset = dst.as_ptr().align_offset(Self::REQUIRED_ALIGNMENT);
        let offset_dst = dst.get_mut(offset..)?;
        let len = Self::try_serialize_into(bitmap, offset_dst)?;
        Some(&mut dst[offset..offset + len])
    }

    /// Serialize a bitmap into bytes, using the provided buffer to store the serialized data
    ///
    /// This method does not require the buffer to be aligned, and will return `None` if the buffer
    /// is not large enough to store the serialized data. Some serializers may have other
    /// conditions where this method will return `None`, for example the [`crate::Frozen`] format
    /// requires the bitmap to be shrunk with [`Bitmap64::shrink_to_fit`] before it can be
    /// serialized.
    ///
    /// This is a niche method, and is not recommended for general use. The
    /// [`Bitmap64::try_serialize_into`]/[`Bitmap64::serialize_into_vec`] methods should usually be used
    /// instead of this method.
    fn try_serialize_into(bitmap: &Bitmap64, dst: &mut [u8]) -> Option<usize> {
        let required_len = Self::get_serialized_size_in_bytes(bitmap);
        if dst.len() < required_len || required_len == 0 {
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
    fn get_serialized_size_in_bytes(bitmap: &Bitmap64) -> usize;

    #[doc(hidden)]
    unsafe fn raw_serialize(bitmap: &Bitmap64, dst: *mut c_char);
}

/// Trait for different formats of bitmap deserialization
pub trait Deserializer: crate::sealed::Sealed {
    /// Try to deserialize a bitmap from the beginning of the provided buffer
    ///
    /// The [`Bitmap64::try_deserialize`] method should usually be used instead of this method
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
    fn try_deserialize(buffer: &[u8]) -> Option<Bitmap64>;

    /// Deserialize a bitmap from the beginning of the provided buffer
    ///
    /// # Safety
    ///
    /// Unlike its safe counterpart, [`Self::try_deserialize`], this function assumes the data is valid,
    /// passing data which does not contain/start with a bitmap serialized with this format will
    /// result in undefined behavior.
    unsafe fn try_deserialize_unchecked(buffer: &[u8]) -> Bitmap64;

    /// Find the end of a serialized bitmap in portable format
    ///
    /// Returns the number of bytes in the buffer which are part of the serialized bitmap, or `None` if
    /// the buffer does not start with a valid serialized bitmap.
    fn find_end(buffer: &[u8]) -> Option<NonZeroUsize>;
}

pub trait ViewDeserializer: crate::sealed::Sealed {
    /// Create a bitmap64 view using the passed data
    ///
    /// # Safety
    /// * `data` must be the result of serializing a roaring bitmap in this format.
    /// * Its beginning must be aligned properly for this format.
    /// * data.len() must be equal exactly to the size of the serialized bitmap.
    ///
    /// See [`Bitmap64View::deserialize`] for examples.
    #[doc(hidden)]
    unsafe fn deserialize_view(data: &[u8]) -> *mut ffi::roaring64_bitmap_t;
}

impl Serializer for Portable {
    /// Computes the serialized size in bytes of the Bitmap in portable format.
    /// See [`Bitmap64::get_serialized_size_in_bytes`] for examples.
    #[doc(alias = "roaring64_bitmap_portable_size_in_bytes")]
    fn get_serialized_size_in_bytes(bitmap: &Bitmap64) -> usize {
        unsafe { ffi::roaring64_bitmap_portable_size_in_bytes(bitmap.raw.as_ptr()) }
    }

    unsafe fn raw_serialize(bitmap: &Bitmap64, dst: *mut c_char) {
        unsafe {
            ffi::roaring64_bitmap_portable_serialize(bitmap.raw.as_ptr(), dst);
        }
    }
}

impl Deserializer for Portable {
    #[doc(alias = "roaring64_bitmap_portable_deserialize_safe")]
    fn try_deserialize(buffer: &[u8]) -> Option<Bitmap64> {
        let raw = unsafe {
            ffi::roaring64_bitmap_portable_deserialize_safe(buffer.as_ptr().cast(), buffer.len())
        };
        if raw.is_null() {
            return None;
        }

        unsafe {
            let bitmap = Bitmap64::take_heap(raw);
            if bitmap.internal_validate().is_ok() {
                Some(bitmap)
            } else {
                None
            }
        }
    }

    unsafe fn try_deserialize_unchecked(buffer: &[u8]) -> Bitmap64 {
        Self::try_deserialize(buffer).unwrap_unchecked()
    }

    fn find_end(buffer: &[u8]) -> Option<NonZeroUsize> {
        let end = unsafe {
            ffi::roaring64_bitmap_portable_deserialize_size(
                buffer.as_ptr().cast::<c_char>(),
                buffer.len(),
            )
        };
        NonZeroUsize::new(end)
    }
}

impl Serializer for Frozen {
    // Unlike 32 bit bitmaps, 64 bit bitmaps require 64 byte alignment
    const REQUIRED_ALIGNMENT: usize = 64;

    fn get_serialized_size_in_bytes(bitmap: &Bitmap64) -> usize {
        unsafe { ffi::roaring64_bitmap_frozen_size_in_bytes(bitmap.raw.as_ptr()) }
    }

    unsafe fn raw_serialize(bitmap: &Bitmap64, dst: *mut c_char) {
        unsafe {
            ffi::roaring64_bitmap_frozen_serialize(bitmap.raw.as_ptr(), dst);
        }
    }
}

impl ViewDeserializer for Frozen {
    unsafe fn deserialize_view(data: &[u8]) -> *mut ffi::roaring64_bitmap_t {
        unsafe { ffi::roaring64_bitmap_frozen_view(data.as_ptr().cast(), data.len()) }
    }
}
