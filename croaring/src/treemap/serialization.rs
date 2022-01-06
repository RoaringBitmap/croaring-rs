use crate::Bitmap;
use crate::Treemap;

use std::io::{Cursor, Result, Seek, SeekFrom};
use std::mem::size_of;
use byteorder::{NativeEndian, BigEndian, ReadBytesExt, WriteBytesExt};

pub trait Serializer {}

/// croaring::Treemap serializer that is compatible with C++ version found in
/// CRoaring at https://github.com/RoaringBitmap/CRoaring/blob/master/cpp/roaring64map.hh
pub trait NativeSerializer: Serializer {
    type Item;

    fn serialize(&self) -> Result<Vec<u8>>;
    fn deserialize(buffer: &[u8]) -> Result<Self::Item>;
    fn get_serialized_size_in_bytes(&self) -> usize;
}

impl Serializer for Treemap {}

impl NativeSerializer for Treemap {
    type Item = Treemap;

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut buffer = vec![];
        buffer.write_u64::<NativeEndian>(self.map.len() as u64)?;

        for (index, bitmap) in &self.map {
            buffer.write_u32::<NativeEndian>(*index)?;
            let bitmap_buffer = bitmap.serialize();
            buffer.extend(bitmap_buffer);
        }

        Ok(buffer)
    }

    fn deserialize(buffer: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(&buffer);
        let mut treemap = Treemap::create();
        let bitmap_count = cursor.read_u64::<NativeEndian>()?;

        for _ in 0..bitmap_count {
            let index = cursor.read_u32::<NativeEndian>()?;
            let bitmap = Bitmap::deserialize(
                &buffer[cursor.position() as usize..]
            );
            cursor.seek(
                SeekFrom::Current(bitmap.get_serialized_size_in_bytes() as i64)
            )?;
            treemap.map.insert(index, bitmap);
        }

        Ok(treemap)
    }

    /// How many bytes are required to serialize this bitmap with
    /// NativeSerializer
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Treemap;
    /// use croaring::treemap::NativeSerializer;
    ///
    /// let mut treemap = Treemap::create();
    ///
    /// for i in 100..1000 {
    ///   treemap.add(i);
    /// }
    ///
    /// treemap.add(std::u32::MAX as u64);
    /// treemap.add(std::u64::MAX);
    ///
    /// assert_eq!(treemap.get_serialized_size_in_bytes(), 1860);
    /// ```
    fn get_serialized_size_in_bytes(&self) -> usize {
        self.map.iter().fold(
            size_of::<u64>() + self.map.len() * size_of::<u32>(),
            |sum, (_, bitmap)| sum + bitmap.get_serialized_size_in_bytes()
        )
    }
}

/// croaring::Treemap serializer that is compatible with JVM version of Treemap
/// found in RoaringBitmap Java implementation at:
/// https://github.com/RoaringBitmap/RoaringBitmap/blob/master/roaringbitmap/src/main/java/org/roaringbitmap/longlong/Roaring64NavigableMap.java
pub trait JvmSerializer: Serializer {
    type Item;

    fn serialize(&self) -> Result<Vec<u8>>;
    fn deserialize(buffer: &[u8]) -> Result<Self::Item>;
    fn get_serialized_size_in_bytes(&self) -> usize;
}

impl JvmSerializer for Treemap {
    type Item = Treemap;

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut buffer = vec![];
        buffer.write_u8(0)?;
        buffer.write_u32::<BigEndian>(self.map.len() as u32)?;

        for (index, bitmap) in &self.map {
            buffer.write_u32::<BigEndian>(*index)?;
            let bitmap_buffer = bitmap.serialize();
            buffer.extend(bitmap_buffer);
        }

        Ok(buffer)
    }

    fn deserialize(buffer: &[u8]) -> Result<Self::Item> {
        let mut cursor = Cursor::new(&buffer);
        cursor.read_u8()?; // read and discard boolean indicator

        let mut treemap = Treemap::create();
        let bitmap_count = cursor.read_u32::<BigEndian>()?;

        for _ in 0..bitmap_count {
            let index = cursor.read_u32::<BigEndian>()?;
            let bitmap = Bitmap::deserialize(
                &buffer[cursor.position() as usize..]
            );
            cursor.seek(
                SeekFrom::Current(bitmap.get_serialized_size_in_bytes() as i64)
            )?;
            treemap.map.insert(index, bitmap);
        }

        Ok(treemap)
    }

    /// How many bytes are required to serialize this bitmap with
    /// JvmSerializer
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Treemap;
    /// use croaring::treemap::JvmSerializer;
    ///
    /// let mut treemap = Treemap::create();
    ///
    /// for i in 100..1000 {
    ///   treemap.add(i);
    /// }
    ///
    /// treemap.add(std::u32::MAX as u64);
    /// treemap.add(std::u64::MAX);
    ///
    /// assert_eq!(treemap.get_serialized_size_in_bytes(), 1857);
    /// ```
    fn get_serialized_size_in_bytes(&self) -> usize {
        self.map.iter().fold(
            size_of::<u8>() + size_of::<u32>() + self.map.len() * size_of::<u32>(),
            |sum, (_, bitmap)| sum + bitmap.get_serialized_size_in_bytes()
        )
    }
}
