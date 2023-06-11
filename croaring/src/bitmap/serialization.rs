use super::Bitmap;

use std::ffi::{c_char, c_void};

pub struct PortableSerializer {}

impl PortableSerializer {
    pub fn serialize_into<'a>(bitmap: &Bitmap, dst: &'a mut Vec<u8>) -> &'a [u8] {
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

    pub fn get_serialized_size_in_bytes(bitmap: &Bitmap) -> usize {
        unsafe { ffi::roaring_bitmap_portable_size_in_bytes(&bitmap.bitmap) }
    }

    pub fn try_deserialize(buffer: &[u8]) -> Option<Bitmap> {
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

pub struct NativeSerializer {}

impl NativeSerializer {
    pub fn serialize_into<'a>(bitmap: &Bitmap, dst: &'a mut Vec<u8>) -> &'a [u8] {
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

    pub fn get_serialized_size_in_bytes(bitmap: &Bitmap) -> usize {
        unsafe { ffi::roaring_bitmap_size_in_bytes(&bitmap.bitmap) }
    }

    pub fn try_deserialize(buffer: &[u8]) -> Option<Bitmap> {
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

pub struct FrozenSerializer {}

impl FrozenSerializer {
    pub fn serialize_into<'a>(bitmap: &Bitmap, dst: &'a mut Vec<u8>) -> &'a [u8] {
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

    pub fn get_serialized_size_in_bytes(bitmap: &Bitmap) -> usize {
        unsafe { ffi::roaring_bitmap_frozen_size_in_bytes(&bitmap.bitmap) }
    }
}
