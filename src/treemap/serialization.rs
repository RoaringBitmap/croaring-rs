use Bitmap;
use Treemap;

use std::io::{Cursor, Result, Seek, SeekFrom};
use byteorder::{NativeEndian, BigEndian, ReadBytesExt, WriteBytesExt};

pub trait Serializer {}

/// croaring::Treemap serializer that is compatible with C++ version found in
/// CRoaring at https://github.com/RoaringBitmap/CRoaring/blob/master/cpp/roaring64map.hh
pub trait NativeSerializer: Serializer {
    type Item;

    fn serialize(&self) -> Result<Vec<u8>>;
    fn deserialize(buffer: &[u8]) -> Result<Self::Item>;
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
}

/// croaring::Treemap serializer that is compatible with JVM version of Treemap
/// found in RoaringBitmap Java implementation at:
/// https://github.com/RoaringBitmap/RoaringBitmap/blob/master/roaringbitmap/src/main/java/org/roaringbitmap/longlong/Roaring64NavigableMap.java
pub trait JvmSerializer: Serializer {
    type Item;

    fn serialize(&self) -> Result<Vec<u8>>;
    fn deserialize(buffer: &[u8]) -> Result<Self::Item>;
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
}
