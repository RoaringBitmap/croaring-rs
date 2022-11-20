use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Result};
use std::u32;

use croaring::{Bitmap, Treemap};
use proptest::prelude::*;

// borrowed and adapted from https://github.com/Nemo157/roaring-rs/blob/5089f180ca7e17db25f5c58023f4460d973e747f/tests/lib.rs#L7-L37
#[test]
fn smoke1() {
    let mut bitmap = Bitmap::create();
    assert_eq!(bitmap.cardinality(), 0);
    assert!(bitmap.is_empty());
    bitmap.remove(0);
    assert_eq!(bitmap.cardinality(), 0);
    assert!(bitmap.is_empty());
    bitmap.add(1);
    assert!(bitmap.contains(1));
    assert_eq!(bitmap.cardinality(), 1);
    assert!(!bitmap.is_empty());
    bitmap.add(u32::MAX - 2);
    assert!(bitmap.contains(u32::MAX - 2));
    assert_eq!(bitmap.cardinality(), 2);
    bitmap.add(u32::MAX);
    assert!(bitmap.contains(u32::MAX));
    assert_eq!(bitmap.cardinality(), 3);
    bitmap.add(2);
    assert!(bitmap.contains(2));
    assert_eq!(bitmap.cardinality(), 4);
    bitmap.remove(2);
    assert!(!bitmap.contains(2));
    assert_eq!(bitmap.cardinality(), 3);
    assert!(!bitmap.contains(0));
    assert!(bitmap.contains(1));
    assert!(!bitmap.contains(100));
    assert!(bitmap.contains(u32::MAX - 2));
    assert!(!bitmap.contains(u32::MAX - 1));
    assert!(bitmap.contains(u32::MAX));
    bitmap.clear();
    assert_eq!(bitmap.cardinality(), 0);
    assert!(bitmap.is_empty());
}

// borrowed and adapted from https://github.com/Bitmap/gocroaring/blob/4a2fc02f79b1c36b904301e7d052f7f0017b6973/gocroaring_test.go#L24-L64
#[test]
fn smoke2() {
    let mut rb1 = Bitmap::create();
    rb1.add(1);
    rb1.add(2);
    rb1.add(3);
    rb1.add(4);
    rb1.add(5);
    rb1.add(100);
    rb1.add(1000);
    rb1.run_optimize();

    let mut rb2 = Bitmap::create();
    rb2.add(3);
    rb2.add(4);
    rb2.add(1000);
    rb2.run_optimize();

    let mut rb3 = Bitmap::create();

    assert_eq!(rb1.cardinality(), 7);
    assert!(rb1.contains(3));

    rb1.and_inplace(&rb2);
    rb3.add(5);
    rb3.or_inplace(&rb1);

    let mut rb4 = Bitmap::fast_or(&[&rb1, &rb2, &rb3]);

    rb1.and_inplace(&rb2);
    println!("{:?}", rb1);

    rb3.add(5);
    rb3.or_inplace(&rb1);

    println!("{:?}", rb1);

    rb3.add(5);
    rb3.or_inplace(&rb1);

    println!("{:?}", rb3.to_vec());
    println!("{:?}", rb3);
    println!("{:?}", rb4);

    rb4 = Bitmap::fast_or(&[&rb1, &rb2, &rb3]);

    println!("{:?}", rb4);
}

fn read_file(path: &str) -> Result<Vec<u8>> {
    let mut bitmap_file = File::open(path)?;
    let file_metadata = bitmap_file.metadata()?;
    let mut buffer = Vec::with_capacity(file_metadata.len() as usize);
    bitmap_file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

#[test]
fn test_treemap_deserialize_cpp() {
    match read_file("tests/data/testcpp.bin") {
        Ok(buffer) => {
            use croaring::treemap::NativeSerializer;

            let treemap = Treemap::deserialize(&buffer).unwrap();

            for i in 100..1000 {
                assert!(treemap.contains(i));
            }

            assert!(treemap.contains(std::u32::MAX as u64));
            assert!(treemap.contains(std::u64::MAX));
        }
        Err(err) => panic!("Cannot read test file {}", err),
    }
}

#[test]
fn test_treemap_deserialize_jvm() {
    match read_file("tests/data/testjvm.bin") {
        Ok(buffer) => {
            use croaring::treemap::JvmSerializer;

            let treemap = Treemap::deserialize(&buffer).unwrap();

            for i in 100..1000 {
                assert!(treemap.contains(i));
            }

            assert!(treemap.contains(std::u32::MAX as u64));
            assert!(treemap.contains(std::u64::MAX));
        }
        Err(err) => panic!("Cannot read test file {}", err),
    }
}

#[test]
fn treemap_run_optimized() {

    use croaring::treemap::JvmSerializer;

    let mut initial = Bitmap::create();
    initial.add(1);
    initial.add(2);
    initial.add(3);
    initial.add(4);
    initial.add(5);
    initial.add(100);
    initial.add(1000);
    let optimized = {
        let mut result = initial.clone();
        result.run_optimize();
        result
    };

    let tree_unoptimized = Treemap { map: BTreeMap::from([(1, initial.clone()), (2, initial)])};
    let tree_optimized = Treemap { map: BTreeMap::from([(1, optimized.clone()), (2, optimized)])};

    let mut test = tree_unoptimized.clone();
    test.run_optimize();
    assert_eq!(test.get_serialized_size_in_bytes(), tree_optimized.get_serialized_size_in_bytes());
    test.remove_run_compression();
    assert_eq!(test.get_serialized_size_in_bytes(), tree_unoptimized.get_serialized_size_in_bytes());
}

proptest! {
    #[test]
    fn bitmap_cardinality_roundtrip(
        indices in prop::collection::vec(proptest::num::u32::ANY, 1..3000)
    ) {
        let original = Bitmap::of(&indices);
        let mut a = indices;
        a.sort_unstable();
        a.dedup();
        prop_assert_eq!(a.len(), original.cardinality() as usize);
    }

    #[test]
    fn treemap_cardinality_roundtrip(
        indices in prop::collection::vec(proptest::num::u64::ANY, 1..3000)
    ) {
        let original = Treemap::of(&indices);
        let mut a = indices;
        a.sort_unstable();
        a.dedup();
        prop_assert_eq!(a.len(), original.cardinality() as usize);
    }

    #[test]
    fn test_bitmap_serialization_roundtrip(
        indices in prop::collection::vec(proptest::num::u32::ANY, 1..3000)
    ) {
        let original = Bitmap::of(&indices);

        let buffer = original.serialize();

        let deserialized = Bitmap::deserialize(&buffer);

        prop_assert_eq!(original , deserialized);
    }

    #[test]
    fn test_treemap_native_serialization_roundtrip(
        indices in prop::collection::vec(proptest::num::u64::ANY, 1..3000)
    ) {
        use croaring::treemap::NativeSerializer;

        let original = Treemap::of(&indices);

        let buffer = original.serialize().unwrap();

        let deserialized = Treemap::deserialize(&buffer).unwrap();

        prop_assert_eq!(original , deserialized);
    }

    #[test]
    fn test_treemap_jvm_serialization_roundtrip(
        indices in prop::collection::vec(proptest::num::u64::ANY, 1..3000)
    ) {
        use croaring::treemap::JvmSerializer;

        let original = Treemap::of(&indices);

        let buffer = original.serialize().unwrap();

        let deserialized = Treemap::deserialize(&buffer).unwrap();

        prop_assert_eq!(original , deserialized);
    }
}
