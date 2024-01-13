use std::collections::BTreeMap;
use std::{fs, iter, u32};

use croaring::{Bitmap, BitmapView, Frozen, JvmLegacy, Native, Portable, Treemap};
use proptest::prelude::*;

// borrowed and adapted from https://github.com/Nemo157/roaring-rs/blob/5089f180ca7e17db25f5c58023f4460d973e747f/tests/lib.rs#L7-L37
#[test]
fn smoke1() {
    let mut bitmap = Bitmap::new();
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
    let mut rb1 = Bitmap::new();
    rb1.add(1);
    rb1.add(2);
    rb1.add(3);
    rb1.add(4);
    rb1.add(5);
    rb1.add(100);
    rb1.add(1000);
    rb1.run_optimize();

    let mut rb2 = Bitmap::new();
    rb2.add(3);
    rb2.add(4);
    rb2.add(1000);
    rb2.run_optimize();

    let mut rb3 = Bitmap::new();

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

fn expected_serialized_bitmap() -> Bitmap {
    let mut bitmap = Bitmap::new();
    bitmap.add_range(0x0_0000..0x0_9000);
    bitmap.add_range(0x0_A000..0x1_0000);
    bitmap.add(0x2_0000);
    bitmap.add(0x2_0005);
    for i in (0x8_0000..0x9_0000).step_by(2) {
        bitmap.add(i);
    }
    bitmap
}

#[test]
fn test_portable_view() {
    let buffer = fs::read("tests/data/portable_bitmap.bin").unwrap();
    let bitmap = unsafe { BitmapView::deserialize::<Portable>(&buffer) };
    let expected = expected_serialized_bitmap();
    assert_eq!(bitmap, expected);
    assert!(bitmap.iter().eq(expected.iter()))
}

#[test]
fn test_native() {
    let buffer = fs::read("tests/data/native_bitmap.bin").unwrap();
    let bitmap = Bitmap::deserialize::<Native>(&buffer);
    let expected = expected_serialized_bitmap();
    assert_eq!(bitmap, expected);
    assert!(bitmap.iter().eq(expected.iter()))
}

#[test]
fn test_frozen_view() {
    let mut buffer = fs::read("tests/data/frozen_bitmap.bin").unwrap();
    // Ensure inserting zeros won't move the data
    buffer.reserve(32);
    let offset = 32 - (buffer.as_ptr() as usize) % 32;
    buffer.splice(..0, iter::repeat(0).take(offset));

    let bitmap = unsafe { BitmapView::deserialize::<Frozen>(&buffer[offset..]) };
    let expected = expected_serialized_bitmap();
    assert_eq!(bitmap, expected);
    assert!(bitmap.iter().eq(expected.iter()))
}

#[test]
fn test_treemap_deserialize_cpp() {
    match fs::read("tests/data/testcpp.bin") {
        Ok(buffer) => {
            let treemap = Treemap::try_deserialize::<Portable>(&buffer).unwrap();

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
    match fs::read("tests/data/testjvm.bin") {
        Ok(buffer) => {
            let treemap = Treemap::try_deserialize::<JvmLegacy>(&buffer).unwrap();

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
fn test_treemap_max_andnot_empty() {
    let single_max = Treemap::of(&[std::u64::MAX]);
    let empty = Treemap::new();
    let diff = single_max.andnot(&empty);
    assert_eq!(diff, single_max);

    let mut diff = single_max.clone();
    diff.andnot_inplace(&empty);
    assert_eq!(diff, single_max);
}

#[test]
fn treemap_run_optimized() {
    let mut initial = Bitmap::new();
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

    let tree_unoptimized = Treemap {
        map: BTreeMap::from([(1, initial.clone()), (2, initial)]),
    };
    let tree_optimized = Treemap {
        map: BTreeMap::from([(1, optimized.clone()), (2, optimized)]),
    };

    let mut test = tree_unoptimized.clone();
    test.run_optimize();
    assert_eq!(
        test.get_serialized_size_in_bytes::<JvmLegacy>(),
        tree_optimized.get_serialized_size_in_bytes::<JvmLegacy>()
    );
    test.remove_run_compression();
    assert_eq!(
        test.get_serialized_size_in_bytes::<JvmLegacy>(),
        tree_unoptimized.get_serialized_size_in_bytes::<JvmLegacy>()
    );
}

#[test]
fn serialize_into_existing_vec_frozen() {
    let mut buffer = vec![0; 13];
    let bitmap = Bitmap::of(&[1, 2, 3, 4, 5]);

    let data = bitmap.serialize_into::<Frozen>(&mut buffer);
    assert_eq!(unsafe { BitmapView::deserialize::<Frozen>(data) }, bitmap);
    assert!(unsafe { data.as_ptr().offset_from(buffer.as_ptr()) } >= 13);
}

#[test]
fn serialize_into_existing_vec_portable() {
    let mut buffer = vec![0; 13];
    let bitmap = Bitmap::of(&[1, 2, 3, 4, 5]);
    let data = bitmap.serialize_into::<Portable>(&mut buffer);
    assert_eq!(Bitmap::try_deserialize::<Portable>(data).unwrap(), bitmap);
    assert!(unsafe { data.as_ptr().offset_from(buffer.as_ptr()) } >= 13);
}

#[test]
fn serialize_into_existing_vec_native() {
    let mut buffer = vec![0; 13];
    let bitmap = Bitmap::of(&[1, 2, 3, 4, 5]);
    let data = bitmap.serialize_into::<Native>(&mut buffer);
    assert_eq!(Bitmap::try_deserialize::<Native>(data).unwrap(), bitmap);
    assert!(unsafe { data.as_ptr().offset_from(buffer.as_ptr()) } >= 13);
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

        let buffer = original.serialize::<Portable>();

        let deserialized = Bitmap::deserialize::<Portable>(&buffer);

        prop_assert_eq!(original , deserialized);
    }

    #[test]
    fn test_treemap_native_serialization_roundtrip(
        indices in prop::collection::vec(proptest::num::u64::ANY, 1..3000)
    ) {
        let original = Treemap::of(&indices);

        let buffer = original.serialize::<Portable>();

        let deserialized = Treemap::try_deserialize::<Portable>(&buffer).unwrap();

        prop_assert_eq!(original , deserialized);
    }

    #[test]
    fn test_treemap_jvm_serialization_roundtrip(
        indices in prop::collection::vec(proptest::num::u64::ANY, 1..3000)
    ) {
        let original = Treemap::of(&indices);

        let buffer = original.serialize::<JvmLegacy>();

        let deserialized = Treemap::try_deserialize::<JvmLegacy>(&buffer).unwrap();

        prop_assert_eq!(original , deserialized);
    }
}

proptest! {
    #[test]
    fn frozen_bitmap_portable_roundtrip(
        indices in prop::collection::vec(proptest::num::u32::ANY, 0..3000)
    ) {
        use croaring::BitmapView;

        let original = Bitmap::of(&indices);
        let serialized = original.serialize::<Portable>();
        let deserialized = unsafe { BitmapView::deserialize::<Portable>(&serialized) };
        assert_eq!(&original, &*deserialized);
        assert!(original.iter().eq(deserialized.iter()));
    }

    #[test]
    fn native_bitmap_roundtrip(
        indices in prop::collection::vec(proptest::num::u32::ANY, 0..3000)
    ) {
        use croaring::Bitmap;

        let original = Bitmap::of(&indices);
        let serialized = original.serialize::<Native>();
        let deserialized = Bitmap::deserialize::<Native>(&serialized[..]);
        assert_eq!(&original, &deserialized);
        assert!(original.iter().eq(deserialized.iter()));
    }

    #[test]
    fn frozen_bitmap_roundtrip(
        indices in prop::collection::vec(proptest::num::u32::ANY, 0..3000)
    ) {
        use croaring::BitmapView;

        let original = Bitmap::of(&indices);
        let mut buf = Vec::new();
        let serialized: &[u8] = original.serialize_into::<Frozen>(&mut buf);
        let deserialized = unsafe { BitmapView::deserialize::<Frozen>(serialized) };
        assert_eq!(&original, &*deserialized);
        assert!(original.iter().eq(deserialized.iter()));
    }
}
