use std::{fs, iter, u32};

use croaring::{Bitmap, BitmapView, Frozen, Native, Portable};
use proptest::prelude::*;

#[cfg(feature = "alloc")]
use croaring::{JvmLegacy, Treemap};

fn init() {
    #[cfg(feature = "alloc")]
    {
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| unsafe { croaring::configure_rust_alloc() });
    }
}

// borrowed and adapted from https://github.com/Nemo157/roaring-rs/blob/5089f180ca7e17db25f5c58023f4460d973e747f/tests/lib.rs#L7-L37
#[test]
fn smoke1() {
    init();
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
    init();
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

    rb1.and_inplace(&rb2);
    println!("{:?}", rb1);

    rb3.add(5);
    rb3.or_inplace(&rb1);

    println!("{:?}", rb1);

    rb3.add(5);
    rb3.or_inplace(&rb1);

    println!("{:?}", rb3);

    #[cfg(feature = "alloc")]
    {
        println!("{:?}", rb3.to_vec());
        let mut rb4 = Bitmap::fast_or(&[&rb1, &rb2, &rb3]);
        println!("{:?}", rb4);

        rb4 = Bitmap::fast_or(&[&rb1, &rb2, &rb3]);
        println!("{:?}", rb4);
    }
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
fn empty_cursor() {
    init();
    let bitmap = Bitmap::new();
    let mut cursor = bitmap.cursor();
    assert!(!cursor.has_value());
    assert_eq!(cursor.current(), None);
    assert_eq!(cursor.prev(), None);
    assert_eq!(cursor.prev(), None);
    assert_eq!(cursor.next(), None);
    assert_eq!(cursor.next(), None);
}

#[test]
fn cursor_return_from_the_edge() {
    init();
    let bitmap = Bitmap::from([1, 2, u32::MAX]);
    let mut cursor = bitmap.cursor_to_last();
    assert_eq!(cursor.current(), Some(u32::MAX));
    assert_eq!(cursor.next(), None);
    assert_eq!(cursor.prev(), Some(u32::MAX));
    assert_eq!(cursor.prev(), Some(2));
    assert_eq!(cursor.prev(), Some(1));

    assert_eq!(cursor.current(), Some(1));
    assert_eq!(cursor.prev(), None);
    assert_eq!(cursor.prev(), None);
    assert_eq!(cursor.next(), Some(1));
}

#[test]
fn test_portable_view() {
    init();
    let mut buffer = fs::read("tests/data/portable_bitmap.bin").unwrap();
    buffer.reserve(32);
    for offset in 0..32 {
        let bitmap = unsafe { BitmapView::deserialize::<Portable>(&buffer[offset..]) };
        let expected = expected_serialized_bitmap();
        assert_eq!(bitmap, expected);
        assert!(bitmap.iter().eq(expected.iter()));
        drop(bitmap);
        buffer.insert(0, 0xff);
    }
}

#[test]
fn test_native() {
    init();
    let mut buffer = fs::read("tests/data/native_bitmap.bin").unwrap();
    buffer.reserve(32);
    for offset in 0..32 {
        let bitmap = Bitmap::deserialize::<Native>(&buffer[offset..]);
        let expected = expected_serialized_bitmap();
        assert_eq!(bitmap, expected);
        assert!(bitmap.iter().eq(expected.iter()));
        buffer.insert(0, 0xff);
    }
}

#[test]
fn test_frozen_view() {
    init();
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
#[cfg(feature = "alloc")]
fn test_treemap_deserialize_cpp() {
    init();
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
#[cfg(feature = "alloc")]
fn test_treemap_deserialize_jvm() {
    init();
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
#[cfg(feature = "alloc")]
fn test_treemap_max_andnot_empty() {
    init();
    let single_max = Treemap::of(&[std::u64::MAX]);
    let empty = Treemap::new();
    let diff = single_max.andnot(&empty);
    assert_eq!(diff, single_max);

    let mut diff = single_max.clone();
    diff.andnot_inplace(&empty);
    assert_eq!(diff, single_max);
}

#[test]
#[cfg(feature = "alloc")]
fn treemap_remove_big_range() {
    init();
    let mut treemap = Treemap::new();
    let value = 0xFFFFFFFFFFFF038D;
    let range_end = 0xFFFFFFFFFF25FFFF;
    treemap.add(value);

    assert!(range_end < value);
    treemap.remove_range(..value);
    assert!(treemap.contains(value));
    assert_eq!(treemap.cardinality(), 1);
}

#[test]
#[cfg(feature = "alloc")]
fn treemap_run_optimized() {
    use std::collections::BTreeMap;
    init();

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
#[cfg(feature = "alloc")]
fn serialize_into_existing_vec_frozen() {
    init();
    let mut buffer = vec![0; 13];
    let bitmap = Bitmap::of(&[1, 2, 3, 4, 5]);

    let data = bitmap.serialize_into_vec::<Frozen>(&mut buffer);
    assert_eq!(unsafe { BitmapView::deserialize::<Frozen>(data) }, bitmap);
    assert!(unsafe { data.as_ptr().offset_from(buffer.as_ptr()) } >= 13);
}

#[test]
#[cfg(feature = "alloc")]
fn serialize_into_existing_vec_norealloc_frozen() {
    init();
    let bitmap = Bitmap::of(&[1, 2, 3, 4, 5]);
    let mut buffer = Vec::with_capacity(
        13 + Frozen::REQUIRED_ALIGNMENT - 1 + bitmap.get_serialized_size_in_bytes::<Frozen>(),
    );
    buffer.resize(13, 1);
    let cap_range = buffer.spare_capacity_mut().as_ptr_range();
    let orig_cap = buffer.capacity();

    let data = bitmap.serialize_into_vec::<Frozen>(&mut buffer);
    assert_eq!(unsafe { BitmapView::deserialize::<Frozen>(data) }, bitmap);
    assert!(cap_range.contains(&data.as_ptr().cast()));
    assert!(unsafe { data.as_ptr().offset_from(cap_range.start.cast()) } < 32);
    assert!(buffer[..13].iter().all(|&b| b == 1));
    assert_eq!(buffer.capacity(), orig_cap);
    assert!(buffer.len() > 13);
}

#[test]
fn serialize_into_existing_slice_presized_aligned_frozen() {
    const SERIALIZED_SIZE: usize = 19;

    #[repr(align(32))]
    struct OverAlign;

    #[repr(C)]
    struct AlignedData {
        _align: OverAlign,
        data: [u8; SERIALIZED_SIZE],
    }
    init();

    let bitmap = Bitmap::of(&[1, 2, 3, 4, 5]);
    let mut buffer = AlignedData {
        _align: OverAlign,
        data: [0; SERIALIZED_SIZE],
    };
    assert_eq!(
        buffer
            .data
            .as_ptr()
            .align_offset(Frozen::REQUIRED_ALIGNMENT),
        0
    );

    let data = bitmap
        .try_serialize_into::<Frozen>(&mut buffer.data)
        .unwrap();
    assert_eq!(unsafe { BitmapView::deserialize::<Frozen>(data) }, bitmap);
    assert_eq!(data.as_ptr_range(), buffer.data.as_ptr_range());
}

#[test]
#[cfg(feature = "alloc")]
fn serialize_into_existing_vec_portable() {
    init();
    let mut buffer = vec![0; 13];
    let bitmap = Bitmap::of(&[1, 2, 3, 4, 5]);
    let data = bitmap.serialize_into_vec::<Portable>(&mut buffer);
    assert_eq!(Bitmap::try_deserialize::<Portable>(data).unwrap(), bitmap);
    assert!(unsafe { data.as_ptr().offset_from(buffer.as_ptr()) } >= 13);
}

#[test]
#[cfg(feature = "alloc")]
fn serialize_into_existing_vec_native() {
    init();
    let mut buffer = vec![0; 13];
    let bitmap = Bitmap::of(&[1, 2, 3, 4, 5]);
    let data = bitmap.serialize_into_vec::<Native>(&mut buffer);
    assert_eq!(Bitmap::try_deserialize::<Native>(data).unwrap(), bitmap);
    assert!(unsafe { data.as_ptr().offset_from(buffer.as_ptr()) } >= 13);
}

proptest! {
    #[test]
    fn bitmap_cardinality_roundtrip(
        indices in prop::collection::vec(proptest::num::u32::ANY, 1..3000)
    ) {
        init();
        let original = Bitmap::of(&indices);
        let mut a = indices;
        a.sort_unstable();
        a.dedup();
        prop_assert_eq!(a.len(), original.cardinality() as usize);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn treemap_cardinality_roundtrip(
        indices in prop::collection::vec(proptest::num::u64::ANY, 1..3000)
    ) {
        init();
        let original = Treemap::of(&indices);
        let mut a = indices;
        a.sort_unstable();
        a.dedup();
        prop_assert_eq!(a.len(), original.cardinality() as usize);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_bitmap_serialization_roundtrip(
        indices in prop::collection::vec(proptest::num::u32::ANY, 1..3000)
    ) {
        init();
        let original = Bitmap::of(&indices);

        let buffer = original.serialize::<Portable>();

        let deserialized = Bitmap::deserialize::<Portable>(&buffer);

        prop_assert_eq!(original , deserialized);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_treemap_native_serialization_roundtrip(
        indices in prop::collection::vec(proptest::num::u64::ANY, 1..3000)
    ) {
        init();
        let original = Treemap::of(&indices);

        let buffer = original.serialize::<Portable>();

        let deserialized = Treemap::try_deserialize::<Portable>(&buffer).unwrap();

        prop_assert_eq!(original , deserialized);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_treemap_jvm_serialization_roundtrip(
        indices in prop::collection::vec(proptest::num::u64::ANY, 1..3000)
    ) {
        init();
        let original = Treemap::of(&indices);

        let buffer = original.serialize::<JvmLegacy>();

        let deserialized = Treemap::try_deserialize::<JvmLegacy>(&buffer).unwrap();

        prop_assert_eq!(original , deserialized);
    }
}

proptest! {
    #[test]
    #[cfg(feature = "alloc")]
    fn frozen_bitmap_portable_roundtrip(
        indices in prop::collection::vec(proptest::num::u32::ANY, 0..3000)
    ) {
        use croaring::BitmapView;
        init();

        let original = Bitmap::of(&indices);
        let serialized = original.serialize::<Portable>();
        let deserialized = unsafe { BitmapView::deserialize::<Portable>(&serialized) };
        assert_eq!(&original, &*deserialized);
        assert!(original.iter().eq(deserialized.iter()));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn native_bitmap_roundtrip(
        indices in prop::collection::vec(proptest::num::u32::ANY, 0..3000)
    ) {
        use croaring::Bitmap;
        init();

        let original = Bitmap::of(&indices);
        let serialized = original.serialize::<Native>();
        let deserialized = Bitmap::deserialize::<Native>(&serialized[..]);
        assert_eq!(&original, &deserialized);
        assert!(original.iter().eq(deserialized.iter()));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn frozen_bitmap_roundtrip(
        indices in prop::collection::vec(proptest::num::u32::ANY, 0..3000)
    ) {
        use croaring::BitmapView;
        init();

        let original = Bitmap::of(&indices);
        let mut buf = Vec::new();
        let serialized: &[u8] = original.serialize_into_vec::<Frozen>(&mut buf);
        let deserialized = unsafe { BitmapView::deserialize::<Frozen>(serialized) };
        assert_eq!(&original, &*deserialized);
        assert!(original.iter().eq(deserialized.iter()));
    }
}

fn expected_words(bitmap: &Bitmap, first_word: u32, n_words: usize) -> Vec<u64> {
    let mut expected = vec![0u64; n_words];
    let start = u64::from(first_word) * 64;
    let end = start + n_words as u64 * 64;
    for value in bitmap.iter() {
        let value = u64::from(value);
        if value >= start && value < end {
            let bit = value - start;
            expected[(bit / 64) as usize] |= 1 << (bit % 64);
        }
    }
    expected
}

#[test]
fn word_range_to_bitset_matches_values() {
    init();
    let bitmap = expected_serialized_bitmap();
    for (first_word, n_words) in [(0, 0), (0, 16), (1000, 300), (2040, 8), (1 << 14, 3000)] {
        let expected = expected_words(&bitmap, first_word, n_words);

        let mut words = vec![u64::MAX; n_words];
        bitmap.word_range_to_bitset(first_word, &mut words);
        assert_eq!(words, expected);

        let mut uninit = Vec::<u64>::with_capacity(n_words);
        let initialized = bitmap
            .word_range_to_bitset_uninit(first_word, &mut uninit.spare_capacity_mut()[..n_words]);
        assert_eq!(initialized, expected);
    }
}

#[test]
fn word_range_to_bitset_end_of_range() {
    init();
    let bitmap = Bitmap::of(&[0, u32::MAX]);
    let mut words = [0u64; 2];
    bitmap.word_range_to_bitset((1 << 26) - 2, &mut words);
    assert_eq!(words, [0, 1 << 63]);
}

#[test]
#[should_panic(expected = "exceeds the 32-bit value range")]
fn word_range_to_bitset_out_of_range() {
    init();
    let mut words = [0u64; 2];
    Bitmap::new().word_range_to_bitset((1 << 26) - 1, &mut words);
}

#[test]
fn word_range_to_bitset_empty_ignores_first_word() {
    init();
    Bitmap::of(&[1]).word_range_to_bitset(u32::MAX, &mut []);
}

const FULL_CARDINALITY: u64 = 1 << 32;

fn full_bitmap() -> Bitmap {
    Bitmap::from_range(0..=u32::MAX)
}

#[test]
fn flip_unbounded_end_includes_max() {
    let flipped = Bitmap::new().flip(..);
    assert_eq!(flipped.cardinality(), FULL_CARDINALITY);
    assert!(flipped.contains(u32::MAX));

    let flipped = Bitmap::new().flip(1..);
    assert_eq!(flipped.cardinality(), FULL_CARDINALITY - 1);
    assert!(flipped.contains(u32::MAX));
}

#[test]
fn flip_inplace_unbounded_end_includes_max() {
    let mut bitmap = Bitmap::new();
    bitmap.flip_inplace(..);
    assert_eq!(bitmap.cardinality(), FULL_CARDINALITY);
    assert!(bitmap.contains(u32::MAX));

    bitmap.flip_inplace(u32::MAX..);
    assert_eq!(bitmap.cardinality(), FULL_CARDINALITY - 1);
    assert!(!bitmap.contains(u32::MAX));
}

#[test]
fn range_cardinality_unbounded_end_includes_max() {
    assert_eq!(full_bitmap().range_cardinality(..), FULL_CARDINALITY);
    assert_eq!(full_bitmap().range_cardinality(1..), FULL_CARDINALITY - 1);

    let bitmap = Bitmap::of(&[u32::MAX]);
    assert_eq!(bitmap.range_cardinality(..), 1);
    assert_eq!(bitmap.range_cardinality(u32::MAX..), 1);
}

#[test]
fn contains_range_unbounded_end_includes_max() {
    assert!(full_bitmap().contains_range(..));
    assert!(full_bitmap().contains_range(5..));

    let mut missing_max = Bitmap::from_range(0..u32::MAX);
    assert!(!missing_max.contains_range(..));
    assert!(!missing_max.contains_range(u32::MAX..));
    missing_max.add(u32::MAX);
    assert!(missing_max.contains_range(..));
}

#[test]
fn intersect_with_range_unbounded_end_includes_max() {
    let bitmap = Bitmap::of(&[u32::MAX]);
    assert!(bitmap.intersect_with_range(..));
    assert!(bitmap.intersect_with_range(u32::MAX..));
    assert!(!bitmap.intersect_with_range(..u32::MAX));
}

#[test]
fn bitset_iter_size_hint_after_exhaustion() {
    init();
    let bitset: croaring::Bitset = [63].into_iter().collect();
    assert_eq!(bitset.size_in_bits(), 64);

    let mut iter = bitset.iter();
    assert_eq!(iter.size_hint(), (0, Some(64)));
    assert_eq!(iter.next(), Some(63));
    assert_eq!(iter.size_hint(), (0, Some(0)));
    for _ in 0..3 {
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }
}

#[test]
fn bitset_iter_size_hint_empty_bitset() {
    init();
    let bitset = croaring::Bitset::new();
    let mut iter = bitset.iter();
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.size_hint(), (0, Some(0)));
}

#[test]
fn bitset_iter_size_hint_after_trailing_unset_bits() {
    init();
    let bitset: croaring::Bitset = [1, 130].into_iter().collect();
    let mut iter = bitset.iter();
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), Some(130));
    for _ in 0..200 {
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }
}

#[test]
fn bitset_self_comparison_when_empty() {
    init();
    let mut allocated_but_clear = croaring::Bitset::new();
    allocated_but_clear.resize_words(2, false);

    for bitset in [croaring::Bitset::new(), allocated_but_clear] {
        assert!(bitset.is_empty());
        assert!(bitset.is_disjoint(&bitset));
        assert!(!bitset.has_intersect(&bitset));
        assert!(bitset.is_superset(&bitset));
        assert_eq!(bitset.intersection_count(&bitset), 0);
        assert_eq!(bitset.union_count(&bitset), 0);
        assert_eq!(bitset.difference_count(&bitset), 0);
        assert_eq!(bitset.symmetric_difference_count(&bitset), 0);
    }
}

#[test]
fn bitset_self_comparison_when_non_empty() {
    init();
    let bitset: croaring::Bitset = [1, 2, 100].into_iter().collect();
    assert!(!bitset.is_disjoint(&bitset));
    assert!(bitset.has_intersect(&bitset));
    assert!(bitset.is_superset(&bitset));
    assert_eq!(bitset.intersection_count(&bitset), 3);
    assert_eq!(bitset.union_count(&bitset), 3);
    assert_eq!(bitset.difference_count(&bitset), 0);
    assert_eq!(bitset.symmetric_difference_count(&bitset), 0);
}
