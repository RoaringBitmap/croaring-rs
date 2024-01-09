use crate::{Bitmap64, Portable};
use std::ffi::c_char;
use std::num::NonZeroUsize;

/// Trait for different formats of bitmap64 serialization
pub trait Serializer {
    /// Serialize a bitmap into bytes, using the provided vec buffer to store the serialized data
    ///
    /// Note that some serializers ([Frozen]) may require that the bitmap is aligned specially,
    /// this method will ensure that the returned slice of bytes is aligned correctly, by adding
    /// additional padding before the serialized data if required.
    ///
    /// The contents of the provided vec buffer will not be overwritten: only new data will be
    /// appended to the end of the buffer. If the buffer has enough capacity, and the current
    /// end of the buffer is correctly aligned, then no additional allocations will be performed.
    fn serialize_into<'a>(bitmap: &Bitmap64, dst: &'a mut Vec<u8>) -> &'a [u8];
    /// Get the number of bytes required to serialize this bitmap
    ///
    /// This does not include any additional padding which may be required to align the bitmap
    fn get_serialized_size_in_bytes(bitmap: &Bitmap64) -> usize;
}

/// Trait for different formats of bitmap deserialization
pub trait Deserializer {
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
    /// Unlike its safe counterpart, [`try_deserialize`], this function assumes the data is valid,
    /// passing data which does not contain/start with a bitmap serialized with this format will
    /// result in undefined behavior.
    unsafe fn try_deserialize_unchecked(buffer: &[u8]) -> Bitmap64;

    /// Find the end of a serialized bitmap in portable format
    ///
    /// Returns the number of bytes in the buffer which are part of the serialized bitmap, or `None` if
    /// the buffer does not start with a valid serialized bitmap.
    fn find_end(buffer: &[u8]) -> Option<NonZeroUsize>;
}

impl Serializer for Portable {
    /// Serialize a bitmap to a slice of bytes in portable format.
    ///
    /// See [`Bitmap64::serialize_into`] for more details.
    #[doc(alias = "roaring64_bitmap_portable_serialize")]
    fn serialize_into<'a>(bitmap: &Bitmap64, dst: &'a mut Vec<u8>) -> &'a [u8] {
        let len = Self::get_serialized_size_in_bytes(bitmap);

        dst.reserve(len);
        let offset = dst.len();
        let total_len = offset.checked_add(len).unwrap();

        unsafe {
            ffi::roaring64_bitmap_portable_serialize(
                bitmap.raw.as_ptr(),
                dst.spare_capacity_mut().as_mut_ptr().cast::<c_char>(),
            );
            dst.set_len(total_len);
        }
        &dst[offset..]
    }

    /// Computes the serialized size in bytes of the Bitmap in portable format.
    /// See [`Bitmap64::get_serialized_size_in_bytes`] for examples.
    #[doc(alias = "roaring64_bitmap_portable_size_in_bytes")]
    fn get_serialized_size_in_bytes(bitmap: &Bitmap64) -> usize {
        unsafe { ffi::roaring64_bitmap_portable_size_in_bytes(bitmap.raw.as_ptr()) }
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
