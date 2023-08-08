use crate::serialization::{Frozen, Native, Portable};
use crate::Treemap;
use crate::{Bitmap, JvmLegacy};
use std::collections::BTreeMap;

use byteorder::{BigEndian, NativeEndian, ReadBytesExt, WriteBytesExt};
use std::mem::size_of;

pub trait Serializer {
    /// Serialize a treemap into bytes, using the provided vec buffer to store the serialized data
    ///
    /// Note that some serializers ([Frozen]) may require that bitmaps are aligned specially, this
    /// method will ensure that the returned slice of bytes is aligned correctly so that each bitmap
    /// is correctly aligned, adding additional padding before the serialized data if required.
    ///
    /// The contents of the provided vec buffer will not be overwritten: only new data will be
    /// appended to the end of the buffer. If the buffer has enough capacity, and the current
    /// end of the buffer is correctly aligned, then no additional allocations will be performed.
    fn serialize_into<'a>(treemap: &Treemap, dst: &'a mut Vec<u8>) -> &'a [u8];

    /// Get the number of bytes required to serialize this bitmap
    ///
    /// This does not include any additional padding which may be required to align the treemap
    fn get_serialized_size_in_bytes(treemap: &Treemap) -> usize;
}

pub trait Deserializer {
    fn try_deserialize(buffer: &[u8]) -> Option<Treemap>;
}

fn serialize_impl<'a, S>(treemap: &Treemap, dst: &'a mut Vec<u8>) -> &'a [u8]
where
    S: crate::bitmap::Serializer,
{
    let start_idx = dst.len();
    let map_len = u64::try_from(treemap.map.len()).unwrap();
    dst.extend_from_slice(&map_len.to_ne_bytes());

    treemap.map.iter().for_each(|(&key, bitmap)| {
        dst.extend_from_slice(&key.to_ne_bytes());
        let prev_len = dst.len();
        let serialized_slice = bitmap.serialize_into::<S>(dst);
        let serialized_len = serialized_slice.len();
        let serialized_range = serialized_slice.as_ptr_range();
        // Serialization should only append the data, no padding can be allowed for this implementation
        debug_assert_eq!(prev_len + serialized_len, dst.len());
        debug_assert_eq!(serialized_range.end, dst.as_ptr_range().end);
    });
    &dst[start_idx..]
}

fn size_in_bytes_impl<S>(treemap: &Treemap) -> usize
where
    S: crate::bitmap::Serializer,
{
    let overhead = size_of::<u64>() + treemap.map.len() * size_of::<u32>();
    let total_sizes = treemap
        .map
        .values()
        .map(|b| b.get_serialized_size_in_bytes::<S>())
        .sum::<usize>();
    overhead + total_sizes
}

fn deserialize_impl<S>(mut buffer: &[u8]) -> Option<Treemap>
where
    S: crate::bitmap::Serializer + crate::bitmap::Deserializer,
{
    let map_len = buffer.read_u64::<NativeEndian>().ok()?;
    let mut map = BTreeMap::new();
    for _ in 0..map_len {
        let key = buffer.read_u32::<NativeEndian>().ok()?;
        let bitmap = Bitmap::try_deserialize::<S>(buffer)?;
        buffer = &buffer[bitmap.get_serialized_size_in_bytes::<S>()..];
        map.insert(key, bitmap);
    }
    Some(Treemap { map })
}

impl Serializer for Portable {
    /// Serializes a Treemap to a slice of bytes in portable format.
    /// See [`Treemap::serialize_into`] for examples.
    fn serialize_into<'a>(treemap: &Treemap, dst: &'a mut Vec<u8>) -> &'a [u8] {
        serialize_impl::<Self>(treemap, dst)
    }

    /// Computes the serialized size in bytes of the Treemap in portable format.
    /// See [`Treemap::get_size_in_bytes`] for examples.
    fn get_serialized_size_in_bytes(treemap: &Treemap) -> usize {
        size_in_bytes_impl::<Self>(treemap)
    }
}

impl Deserializer for Portable {
    fn try_deserialize(buffer: &[u8]) -> Option<Treemap> {
        deserialize_impl::<Self>(buffer)
    }
}

impl Serializer for Native {
    /// Serializes a Treemap to a slice of bytes in native format.
    /// See [`Treemap::serialize_into`] for examples.
    fn serialize_into<'a>(treemap: &Treemap, dst: &'a mut Vec<u8>) -> &'a [u8] {
        serialize_impl::<Self>(treemap, dst)
    }

    /// Computes the serialized size in bytes of the Treemap in native format.
    /// See [`Treemap::get_size_in_bytes`] for examples.
    fn get_serialized_size_in_bytes(treemap: &Treemap) -> usize {
        size_in_bytes_impl::<Self>(treemap)
    }
}

impl Deserializer for Native {
    fn try_deserialize(buffer: &[u8]) -> Option<Treemap> {
        deserialize_impl::<Self>(buffer)
    }
}

impl Serializer for Frozen {
    /// Serializes a Treemap to a slice of bytes in frozen format.
    /// See [`Treemap::serialize_into`] for examples.
    fn serialize_into<'a>(treemap: &Treemap, dst: &'a mut Vec<u8>) -> &'a [u8] {
        const METADATA_SIZE: usize = size_of::<usize>() + size_of::<u32>();

        let len = Self::get_serialized_size_in_bytes(treemap);
        let mut offset = dst.len();
        if dst.capacity() < dst.len() + len
            || (dst.as_ptr() as usize + offset) % Self::REQUIRED_ALIGNMENT != 0
        {
            // Need to be able to add up to 31 extra bytes to align to 32 bytes
            dst.reserve(len.checked_add(Self::REQUIRED_ALIGNMENT - 1).unwrap());
            let extra_offset = match (dst.as_ptr() as usize + offset) % Self::REQUIRED_ALIGNMENT {
                0 => 0,
                r => Self::REQUIRED_ALIGNMENT - r,
            };
            offset = offset.checked_add(extra_offset).unwrap();
            // we must initialize up to offset
            dst.resize(offset, 0);
        }
        let total_len = offset.checked_add(len).unwrap();
        debug_assert!(dst.capacity() >= total_len);

        let map_size = u64::try_from(treemap.map.len()).unwrap();
        dst.extend_from_slice(&map_size.to_be_bytes());

        treemap.map.iter().for_each(|(&key, bitmap)| {
            let extra_padding = match (dst.as_ptr_range().end as usize + METADATA_SIZE)
                % Self::REQUIRED_ALIGNMENT
            {
                0 => 0,
                r => Self::REQUIRED_ALIGNMENT - r,
            };
            dst.resize(dst.len() + extra_padding, 0);

            let frozen_size_in_bytes: usize = bitmap.get_serialized_size_in_bytes::<Self>();
            dst.extend_from_slice(&frozen_size_in_bytes.to_ne_bytes());
            dst.extend_from_slice(&key.to_ne_bytes());

            let before_bitmap_serialize = dst.as_ptr_range().end;
            let serialized_slice = bitmap.serialize_into::<Self>(dst);
            // We pre-calculated padding, so there should be no padding added
            debug_assert_eq!(before_bitmap_serialize, serialized_slice.as_ptr());
            debug_assert_eq!(serialized_slice.as_ptr_range().end, dst.as_ptr_range().end);
        });

        &dst[offset..]
    }

    /// Computes the serialized size in bytes of the Treemap in frozen format.
    /// See [`Treemap::get_size_in_bytes`] for examples.
    fn get_serialized_size_in_bytes(treemap: &Treemap) -> usize {
        // Yes, the frozen format changes based on the size of usize
        const METADATA_SIZE: usize = size_of::<usize>() + size_of::<u32>();

        let mut result = size_of::<u64>();
        for bitmap in treemap.map.values() {
            // pad to 32 bytes minus the metadata size
            result += METADATA_SIZE;
            result += match result % 32 {
                0 => 0,
                r => 32 - r,
            };
            result += bitmap.get_serialized_size_in_bytes::<Self>();
        }
        result
    }
}

impl Serializer for JvmLegacy {
    fn serialize_into<'a>(treemap: &Treemap, dst: &'a mut Vec<u8>) -> &'a [u8] {
        let start_idx = dst.len();
        // Push a boolean false indicating that the values are not signed
        dst.write_u8(0).unwrap();

        let bitmap_count: u32 = treemap.map.len().try_into().unwrap();
        dst.write_u32::<BigEndian>(bitmap_count).unwrap();
        treemap.map.iter().for_each(|(&key, bitmap)| {
            dst.write_u32::<BigEndian>(key).unwrap();
            bitmap.serialize_into::<Portable>(dst);
        });

        &dst[start_idx..]
    }

    fn get_serialized_size_in_bytes(treemap: &Treemap) -> usize {
        let overhead = size_of::<u8>() + size_of::<u32>() + size_of::<u32>() * treemap.map.len();
        let total_sizes = treemap
            .map
            .values()
            .map(|b| b.get_serialized_size_in_bytes::<Portable>())
            .sum::<usize>();
        overhead + total_sizes
    }
}

impl Deserializer for JvmLegacy {
    fn try_deserialize(mut buffer: &[u8]) -> Option<Treemap> {
        // Ignored, we assume that the values are not signed
        let _is_signed = buffer.read_u8().ok()?;

        let bitmap_count = buffer.read_u32::<BigEndian>().ok()?;
        let mut map = BTreeMap::new();
        for _ in 0..bitmap_count {
            let key = buffer.read_u32::<BigEndian>().ok()?;
            let bitmap = Bitmap::try_deserialize::<Portable>(buffer)?;
            buffer = &buffer[bitmap.get_serialized_size_in_bytes::<Portable>()..];
            map.insert(key, bitmap);
        }

        Some(Treemap { map })
    }
}
