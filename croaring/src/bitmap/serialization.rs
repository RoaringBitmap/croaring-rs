use super::{Bitmap, BitmapView};

use std::ffi::{c_char, c_void};

pub trait Serializer {
    fn serialize_into<'a>(bitmap: &Bitmap, dst: &'a mut Vec<u8>) -> &'a [u8];
    fn get_serialized_size_in_bytes(bitmap: &Bitmap) -> usize;
}

pub trait Deserializer {
    fn try_deserialize(buffer: &[u8]) -> Option<Bitmap>;
}

pub trait ViewDeserializer {
    unsafe fn deserialize_view(data: &[u8]) -> BitmapView<'_>;
}

/// The `Portable` format is meant to be compatible with other roaring bitmap libraries, such as Go or Java.
/// It's defined here: https://github.com/RoaringBitmap/RoaringFormatSpec
pub enum Portable {}

impl Serializer for Portable {
    /// Serializes a bitmap to a slice of bytes in portable format.
    /// See [`Bitmap::serialize_into`] for examples.
    #[doc(alias = "roaring_bitmap_portable_serialize")]
    fn serialize_into<'a>(bitmap: &Bitmap, dst: &'a mut Vec<u8>) -> &'a [u8] {
        let len = Self::get_serialized_size_in_bytes(bitmap);

        dst.reserve(len);
        let total_len = dst.len().checked_add(len).unwrap();

        unsafe {
            ffi::roaring_bitmap_portable_serialize(
                &bitmap.bitmap,
                dst.spare_capacity_mut().as_mut_ptr().cast::<c_char>(),
            );
            dst.set_len(total_len);
        }

        dst
    }

    /// Computes the serialized size in bytes of the Bitmap in portable format.
    /// See [`Bitmap::get_size_in_bytes`] for examples.
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
                buffer.as_ptr() as *const c_char,
                buffer.len(),
            );

            if !bitmap.is_null() {
                Some(Bitmap::take_heap(bitmap))
            } else {
                None
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
    unsafe fn deserialize_view<'a>(data: &'a [u8]) -> BitmapView {
        // portable_deserialize_size does some amount of checks, and returns zero if data cannot be valid
        debug_assert_ne!(
            ffi::roaring_bitmap_portable_deserialize_size(data.as_ptr().cast(), data.len()),
            0,
        );
        let roaring = ffi::roaring_bitmap_portable_deserialize_frozen(data.as_ptr().cast());
        BitmapView::take_heap(roaring)
    }
}

/// The `Native` format format can sometimes be more space efficient than [`Portable`], e.g. when
/// the data is sparse. It's not compatible with Java and Go implementations. Use [`Portable`] for
/// that purpose.
pub enum Native {}

impl Serializer for Native {
    /// Serializes a bitmap to a slice of bytes in native format.
    /// See [`Bitmap::serialize_into`] for examples.
    #[doc(alias = "roaring_bitmap_serialize")]
    fn serialize_into<'a>(bitmap: &Bitmap, dst: &'a mut Vec<u8>) -> &'a [u8] {
        let len = Self::get_serialized_size_in_bytes(bitmap);

        dst.reserve(len);
        let total_len = dst.len().checked_add(len).unwrap();

        unsafe {
            ffi::roaring_bitmap_serialize(
                &bitmap.bitmap,
                dst.spare_capacity_mut().as_mut_ptr().cast::<c_char>(),
            );
            dst.set_len(total_len);
        }

        dst
    }

    /// Computes the serialized size in bytes of the Bitmap in native format.
    /// See [`Bitmap::get_size_in_bytes`] for examples.
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
                buffer.as_ptr() as *const c_void,
                buffer.len(),
            );

            if !bitmap.is_null() {
                Some(Bitmap::take_heap(bitmap))
            } else {
                None
            }
        }
    }
}

/// The `Frozen` format imitates memory layout of the underlying C library.
/// This reduces amount of allocations and copying required during deserialization, though
/// `Portable` offers comparable performance.
pub enum Frozen {}

impl Serializer for Frozen {
    /// Serializes a bitmap to a slice of bytes in "frozen" format.
    ///
    /// This has an odd API because it always returns a slice which is aligned to 32 bytes:
    /// This means the returned slice may not start exactly at the beginning of the passed `Vec`
    /// See [`Bitmap::serialize_into`] for examples.
    #[doc(alias = "roaring_bitmap_frozen_serialize")]
    fn serialize_into<'a>(bitmap: &Bitmap, dst: &'a mut Vec<u8>) -> &'a [u8] {
        const REQUIRED_ALIGNMENT: usize = 32;
        let len = Self::get_serialized_size_in_bytes(bitmap);

        let offset = dst.len();
        // Need to be able to add up to 31 extra bytes to align to 32 bytes
        dst.reserve(len.checked_add(REQUIRED_ALIGNMENT - 1).unwrap());

        let extra_offset = match (dst.as_ptr() as usize) % REQUIRED_ALIGNMENT {
            0 => 0,
            r => REQUIRED_ALIGNMENT - r,
        };
        let offset = offset.checked_add(extra_offset).unwrap();
        let total_len = offset.checked_add(len).unwrap();
        debug_assert!(dst.capacity() >= total_len);

        // we must initialize up to offset
        dst.resize(offset, 0);

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
    /// See [`Bitmap::get_size_in_bytes`] for examples.
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
    ///   (in c with `roaring_bitmap_frozen_serialize`, or via [`Bitmap::serialize_frozen_into`]).
    /// * Its beginning must be aligned by 32 bytes.
    /// * data.len() must be equal exactly to the size of the frozen bitmap.
    ///
    /// See [`BitmapView::deserialize`] for examples.
    unsafe fn deserialize_view<'a>(data: &'a [u8]) -> BitmapView {
        const REQUIRED_ALIGNMENT: usize = 32;
        assert_eq!(data.as_ptr() as usize % REQUIRED_ALIGNMENT, 0);

        let roaring = ffi::roaring_bitmap_frozen_view(data.as_ptr().cast(), data.len());
        BitmapView::take_heap(roaring)
    }
}
