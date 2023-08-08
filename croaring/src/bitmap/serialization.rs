use super::{Bitmap, BitmapView};
use crate::serialization::{Frozen, Native, Portable};

use std::ffi::{c_char, c_void};

/// Trait for different formats of bitmap serialization
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
    fn serialize_into<'a>(bitmap: &Bitmap, dst: &'a mut Vec<u8>) -> &'a [u8];
    /// Get the number of bytes required to serialize this bitmap
    ///
    /// This does not include any additional padding which may be required to align the bitmap
    fn get_serialized_size_in_bytes(bitmap: &Bitmap) -> usize;
}

/// Trait for different formats of bitmap deserialization
pub trait Deserializer {
    /// Try to deserialize a bitmap from the beginning of the provided buffer
    ///
    /// If the buffer starts with the serialized representation of a bitmap, then
    /// this method will return a new bitmap containing the deserialized data.
    ///
    /// If the buffer does not start with a serialized bitmap (or contains an invalidly
    /// truncated bitmap), then this method will return `None`.
    ///
    /// To determine how many bytes were consumed from the buffer, use the
    /// [`Serializer::get_serialized_size_in_bytes`] method on the returned bitmap.
    fn try_deserialize(buffer: &[u8]) -> Option<Bitmap>;
}

/// Trait for different formats of bitmap deserialization into a view without copying
pub trait ViewDeserializer {
    /// Create a bitmap view using the passed data
    ///
    /// # Safety
    /// * `data` must be the result of serializing a roaring bitmap in this format.
    /// * Its beginning must be aligned properly for this format.
    /// * data.len() must be equal exactly to the size of the serialized bitmap.
    ///
    /// See [`BitmapView::deserialize`] for examples.
    unsafe fn deserialize_view(data: &[u8]) -> BitmapView<'_>;
}

impl Serializer for Portable {
    /// Serializes a bitmap to a slice of bytes in portable format.
    /// See [`Bitmap::serialize_into`] for examples.
    #[doc(alias = "roaring_bitmap_portable_serialize")]
    fn serialize_into<'a>(bitmap: &Bitmap, dst: &'a mut Vec<u8>) -> &'a [u8] {
        let len = Self::get_serialized_size_in_bytes(bitmap);

        dst.reserve(len);
        let offset = dst.len();
        let total_len = offset.checked_add(len).unwrap();

        unsafe {
            ffi::roaring_bitmap_portable_serialize(
                &bitmap.bitmap,
                dst.spare_capacity_mut().as_mut_ptr().cast::<c_char>(),
            );
            dst.set_len(total_len);
        }

        &dst[offset..]
    }

    /// Computes the serialized size in bytes of the Bitmap in portable format.
    /// See [`Bitmap::get_serialized_size_in_bytes`] for examples.
    #[doc(alias = "roaring_bitmap_portable_size_in_bytes")]
    fn get_serialized_size_in_bytes(bitmap: &Bitmap) -> usize {
        unsafe { ffi::roaring_bitmap_portable_size_in_bytes(&bitmap.bitmap) }
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
                None
            } else {
                Some(Bitmap::take_heap(bitmap))
            }
        }
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

impl Serializer for Native {
    /// Serializes a bitmap to a slice of bytes in native format.
    /// See [`Bitmap::serialize_into`] for examples.
    #[doc(alias = "roaring_bitmap_serialize")]
    fn serialize_into<'a>(bitmap: &Bitmap, dst: &'a mut Vec<u8>) -> &'a [u8] {
        let len = Self::get_serialized_size_in_bytes(bitmap);

        dst.reserve(len);
        let offset = dst.len();
        let total_len = offset.checked_add(len).unwrap();

        unsafe {
            ffi::roaring_bitmap_serialize(
                &bitmap.bitmap,
                dst.spare_capacity_mut().as_mut_ptr().cast::<c_char>(),
            );
            dst.set_len(total_len);
        }

        &dst[offset..]
    }

    /// Computes the serialized size in bytes of the Bitmap in native format.
    /// See [`Bitmap::get_serialized_size_in_bytes`] for examples.
    #[doc(alias = "roaring_bitmap_size_in_bytes")]
    fn get_serialized_size_in_bytes(bitmap: &Bitmap) -> usize {
        unsafe { ffi::roaring_bitmap_size_in_bytes(&bitmap.bitmap) }
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
                None
            } else {
                Some(Bitmap::take_heap(bitmap))
            }
        }
    }
}

impl Serializer for Frozen {
    /// Serializes a bitmap to a slice of bytes in "frozen" format.
    ///
    /// This has an odd API because it always returns a slice which is aligned to 32 bytes:
    /// This means the returned slice may not start exactly at the beginning of the passed `Vec`
    /// See [`Bitmap::serialize_into`] for examples.
    #[doc(alias = "roaring_bitmap_frozen_serialize")]
    fn serialize_into<'a>(bitmap: &Bitmap, dst: &'a mut Vec<u8>) -> &'a [u8] {
        let len = Self::get_serialized_size_in_bytes(bitmap);

        let mut offset = dst.len();
        if dst.capacity() < dst.len() + len
            || (dst.as_ptr_range().end as usize) % Self::REQUIRED_ALIGNMENT != 0
        {
            // Need to be able to add up to 31 extra bytes to align to 32 bytes
            dst.reserve(len.checked_add(Self::REQUIRED_ALIGNMENT - 1).unwrap());
            let extra_offset = match (dst.as_ptr_range().end as usize) % Self::REQUIRED_ALIGNMENT {
                0 => 0,
                r => Self::REQUIRED_ALIGNMENT - r,
            };
            offset = offset.checked_add(extra_offset).unwrap();
            // we must initialize up to offset
            dst.resize(offset, 0);
        }
        let total_len = offset.checked_add(len).unwrap();
        debug_assert!(dst.capacity() >= total_len);

        unsafe {
            ffi::roaring_bitmap_frozen_serialize(
                &bitmap.bitmap,
                dst.as_mut_ptr().add(offset).cast::<c_char>(),
            );
            dst.set_len(total_len);
        }

        &dst[offset..total_len]
    }

    /// Computes the serialized size in bytes of the Bitmap in frozen format.
    /// See [`Bitmap::get_serialized_size_in_bytes`] for examples.
    #[doc(alias = "roaring_bitmap_frozen_size_in_bytes")]
    fn get_serialized_size_in_bytes(bitmap: &Bitmap) -> usize {
        unsafe { ffi::roaring_bitmap_frozen_size_in_bytes(&bitmap.bitmap) }
    }
}

impl ViewDeserializer for Frozen {
    /// Create a frozen bitmap view using the passed data
    ///
    /// # Safety
    /// * `data` must be the result of serializing a roaring bitmap in frozen mode
    ///   (in c with `roaring_bitmap_frozen_serialize`, or via [`Bitmap::serialize_into::<Frozen>`]).
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
