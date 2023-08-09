use crate::serialization::{Frozen, Native, Portable};
use crate::{bitmap, Treemap};
use crate::{Bitmap, JvmLegacy};
use std::collections::BTreeMap;
use std::io;
use std::io::Write as _;

use byteorder::{BigEndian, NativeEndian, ReadBytesExt, WriteBytesExt};
use std::mem::size_of;

pub trait Serializer {
    /// Serialize a treemap into a writer
    ///
    /// Returns the number of bytes written, or an error if writing failed
    ///
    /// Note tha some serializers ([Frozen]) may require that the bitmap is aligned specially when
    /// reading: this method does not perform any extra alignment. See [Self::serialize_into]
    /// for a method which will return a slice of bytes which are guaranteed to be aligned correctly
    /// in memory
    fn serialize_into_writer<W>(treemap: &Treemap, dst: W) -> io::Result<usize>
    where
        W: io::Write;

    /// Serialize a treemap into bytes, using the provided vec buffer to store the serialized data
    ///
    /// Note that some serializers ([Frozen]) may require that bitmaps are aligned specially, this
    /// method will ensure that the returned slice of bytes is aligned correctly so that each bitmap
    /// is correctly aligned, adding additional padding before the serialized data if required.
    ///
    /// The contents of the provided vec buffer will not be overwritten: only new data will be
    /// appended to the end of the buffer. If the buffer has enough capacity, and the current
    /// end of the buffer is correctly aligned, then no additional allocations will be performed.
    ///
    /// Note that this method requires keeping the serialized data in memory: see also the
    /// [`Self::serialize_into_writer`] method which will write the serialized data directly to a
    /// writer
    fn serialize_into<'a>(treemap: &Treemap, dst: &'a mut Vec<u8>) -> &'a [u8];

    /// Get the number of bytes required to serialize this bitmap
    ///
    /// This does not include any additional padding which may be required to align the treemap
    fn get_serialized_size_in_bytes(treemap: &Treemap) -> usize;
}

pub trait Deserializer {
    /// Try to deserialize a treemap from the beginning of the provided buffer
    ///
    /// If the buffer starts with the serialized representation of a treemap, then
    /// this method will return a tuple containing a new treemap containing the deserialized data,
    /// and the number of bytes consumed from the buffer.
    ///
    /// If the buffer does not start with a serialized treemap (or contains an invalidly
    /// truncated treemap), then this method will return `None`.
    fn try_deserialize(buffer: &[u8]) -> Option<(Treemap, usize)>;
}

fn serialize_impl<'a, S>(treemap: &Treemap, dst: &'a mut Vec<u8>) -> &'a [u8]
where
    S: bitmap::Serializer,
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

fn serialize_writer_impl<S, W>(treemap: &Treemap, dst: W) -> io::Result<usize>
where
    S: bitmap::Serializer,
    W: io::Write,
{
    let mut dst = OffsetTrackingWriter::new(dst);
    let map_len = u64::try_from(treemap.map.len()).unwrap();
    dst.write_u64::<NativeEndian>(map_len)?;

    let mut buf = Vec::new();

    for (&key, bitmap) in &treemap.map {
        dst.write_u32::<NativeEndian>(key)?;

        let bitmap_serialized = bitmap.serialize_into::<S>(&mut buf);
        dst.write_all(bitmap_serialized)?;
        buf.clear();
    }
    Ok(dst.bytes_written)
}

fn size_in_bytes_impl<S>(treemap: &Treemap) -> usize
where
    S: bitmap::Serializer,
{
    let overhead = size_of::<u64>() + treemap.map.len() * size_of::<u32>();
    let total_sizes = treemap
        .map
        .values()
        .map(|b| b.get_serialized_size_in_bytes::<S>())
        .sum::<usize>();
    overhead + total_sizes
}

fn deserialize_impl<S>(mut buffer: &[u8]) -> Option<(Treemap, usize)>
where
    S: bitmap::Serializer + bitmap::Deserializer,
{
    let start_len = buffer.len();
    let map_len = buffer.read_u64::<NativeEndian>().ok()?;
    let mut map = BTreeMap::new();
    for _ in 0..map_len {
        let key = buffer.read_u32::<NativeEndian>().ok()?;
        let bitmap = Bitmap::try_deserialize::<S>(buffer)?;
        buffer = &buffer[bitmap.get_serialized_size_in_bytes::<S>()..];
        map.insert(key, bitmap);
    }
    Some((Treemap { map }, start_len - buffer.len()))
}

impl Serializer for Portable {
    /// Serializes a Treemap to a writer in portable format.
    /// See [`Treemap::serialize_into_writer`] for examples.
    fn serialize_into_writer<W>(treemap: &Treemap, dst: W) -> io::Result<usize>
    where
        W: io::Write,
    {
        serialize_writer_impl::<Self, W>(treemap, dst)
    }

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
    fn try_deserialize(buffer: &[u8]) -> Option<(Treemap, usize)> {
        deserialize_impl::<Self>(buffer)
    }
}

impl Serializer for Native {
    /// Serializes a Treemap to a writer in native format.
    /// See [`Treemap::serialize_into_writer`] for examples.
    fn serialize_into_writer<W>(treemap: &Treemap, dst: W) -> io::Result<usize>
    where
        W: io::Write,
    {
        serialize_writer_impl::<Self, W>(treemap, dst)
    }

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
    fn try_deserialize(buffer: &[u8]) -> Option<(Treemap, usize)> {
        deserialize_impl::<Self>(buffer)
    }
}

const FROZEN_BITMAP_METADATA_SIZE: usize = size_of::<usize>() + size_of::<u32>();

impl Serializer for Frozen {
    /// Serializes a Treemap to a writer in frozen format.
    /// See [`Treemap::serialize_into_writer`] for examples.
    fn serialize_into_writer<W>(treemap: &Treemap, dst: W) -> io::Result<usize>
    where
        W: io::Write,
    {
        const MAX_PADDING: usize = Frozen::REQUIRED_ALIGNMENT - 1;
        const FULL_PADDING: [u8; MAX_PADDING] = [0; MAX_PADDING];

        let mut dst = OffsetTrackingWriter::new(dst);

        let map_size = u64::try_from(treemap.map.len()).unwrap();
        dst.write_all(&u64::to_ne_bytes(map_size))?;

        let mut buf = Vec::new();
        for (&key, bitmap) in &treemap.map {
            let bitmap_serialized = bitmap.serialize_into::<Self>(&mut buf);
            let required_padding =
                frozen_needed_alignment(dst.bytes_written + FROZEN_BITMAP_METADATA_SIZE);

            dst.write_all(&FULL_PADDING[..required_padding])?;
            dst.write_all(&usize::to_ne_bytes(bitmap_serialized.len()))?;
            dst.write_all(&u32::to_ne_bytes(key))?;

            debug_assert_eq!(dst.bytes_written % Self::REQUIRED_ALIGNMENT, 0);
            dst.write_all(bitmap_serialized)?;

            buf.clear();
        }

        Ok(dst.bytes_written)
    }

    /// Serializes a Treemap to a slice of bytes in frozen format.
    /// See [`Treemap::serialize_into`] for examples.
    fn serialize_into<'a>(treemap: &Treemap, dst: &'a mut Vec<u8>) -> &'a [u8] {
        let len = Self::get_serialized_size_in_bytes(treemap);
        let mut offset = dst.len();
        if dst.capacity() < dst.len() + len
            || (dst.as_ptr() as usize + offset) % Self::REQUIRED_ALIGNMENT != 0
        {
            // Need to be able to add up to 31 extra bytes to align to 32 bytes
            dst.reserve(len.checked_add(Self::REQUIRED_ALIGNMENT - 1).unwrap());
            let extra_offset = frozen_needed_alignment(dst.as_ptr() as usize + offset);
            offset = offset.checked_add(extra_offset).unwrap();
            // we must initialize up to offset
            dst.resize(offset, 0);
        }
        let total_len = offset.checked_add(len).unwrap();
        debug_assert!(dst.capacity() >= total_len);

        let map_size = u64::try_from(treemap.map.len()).unwrap();
        dst.extend_from_slice(&map_size.to_ne_bytes());

        treemap.map.iter().for_each(|(&key, bitmap)| {
            let end_with_metadata = dst.as_ptr_range().end as usize + FROZEN_BITMAP_METADATA_SIZE;
            let extra_padding = frozen_needed_alignment(end_with_metadata);
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
    fn serialize_into_writer<W>(treemap: &Treemap, dst: W) -> io::Result<usize>
    where
        W: io::Write,
    {
        let mut dst = OffsetTrackingWriter::new(dst);
        // Push a boolean false indicating that the values are not signed
        dst.write_u8(0)?;

        let bitmap_count: u32 = treemap.map.len().try_into().unwrap();
        dst.write_u32::<BigEndian>(bitmap_count)?;

        let mut buf = Vec::new();
        for (&key, bitmap) in &treemap.map {
            dst.write_u32::<BigEndian>(key)?;
            let bitmap_serialized = bitmap.serialize_into::<Portable>(&mut buf);
            dst.write_all(bitmap_serialized)?;
            buf.clear();
        }

        Ok(dst.bytes_written)
    }

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
    fn try_deserialize(mut buffer: &[u8]) -> Option<(Treemap, usize)> {
        let start_len = buffer.len();
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

        Some((Treemap { map }, start_len - buffer.len()))
    }
}

#[inline]
fn frozen_needed_alignment(x: usize) -> usize {
    match x % Frozen::REQUIRED_ALIGNMENT {
        0 => 0,
        r => Frozen::REQUIRED_ALIGNMENT - r,
    }
}

struct OffsetTrackingWriter<W> {
    writer: W,
    bytes_written: usize,
}

impl<W> OffsetTrackingWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            bytes_written: 0,
        }
    }
}

impl<W: io::Write> io::Write for OffsetTrackingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = self.writer.write(buf)?;
        self.bytes_written += written;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.writer.write_all(buf)?;
        self.bytes_written += buf.len();
        Ok(())
    }
}
