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

pub enum Portable {}

impl Serializer for Portable {
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

    fn get_serialized_size_in_bytes(bitmap: &Bitmap) -> usize {
        unsafe { ffi::roaring_bitmap_portable_size_in_bytes(&bitmap.bitmap) }
    }
}

impl Deserializer for Portable {
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

pub enum Native {}

impl Serializer for Native {
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

    fn get_serialized_size_in_bytes(bitmap: &Bitmap) -> usize {
        unsafe { ffi::roaring_bitmap_size_in_bytes(&bitmap.bitmap) }
    }
}

impl Deserializer for Native {
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

pub enum Frozen {}

impl Serializer for Frozen {
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
    unsafe fn deserialize_view<'a>(data: &'a [u8]) -> BitmapView {
        const REQUIRED_ALIGNMENT: usize = 32;
        assert_eq!(data.as_ptr() as usize % REQUIRED_ALIGNMENT, 0);

        let roaring = ffi::roaring_bitmap_frozen_view(data.as_ptr().cast(), data.len());
        BitmapView::take_heap(roaring)
    }
}
