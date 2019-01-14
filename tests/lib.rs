extern crate croaring;
#[cfg(test)]
extern crate quickcheck;
extern crate rand;

use croaring::Bitmap;
use quickcheck::{QuickCheck, StdGen};
use std::u32;

// borrowed and adapted from https://github.com/Nemo157/roaring-rs/blob/5089f180ca7e17db25f5c58023f4460d973e747f/tests/lib.rs#L7-L37
#[test]
fn smoke1() {
    let mut bitmap = Bitmap::create();
    assert_eq!(bitmap.cardinality(), 0);
    assert_eq!(bitmap.is_empty(), true);
    bitmap.remove(0);
    assert_eq!(bitmap.cardinality(), 0);
    assert_eq!(bitmap.is_empty(), true);
    bitmap.add(1);
    assert_eq!(bitmap.contains(1), true);
    assert_eq!(bitmap.cardinality(), 1);
    assert_eq!(bitmap.is_empty(), false);
    bitmap.add(u32::MAX - 2);
    assert_eq!(bitmap.contains(u32::MAX - 2), true);
    assert_eq!(bitmap.cardinality(), 2);
    bitmap.add(u32::MAX);
    assert_eq!(bitmap.contains(u32::MAX), true);
    assert_eq!(bitmap.cardinality(), 3);
    bitmap.add(2);
    assert_eq!(bitmap.contains(2), true);
    assert_eq!(bitmap.cardinality(), 4);
    bitmap.remove(2);
    assert_eq!(bitmap.contains(2), false);
    assert_eq!(bitmap.cardinality(), 3);
    assert_eq!(bitmap.contains(0), false);
    assert_eq!(bitmap.contains(1), true);
    assert_eq!(bitmap.contains(100), false);
    assert_eq!(bitmap.contains(u32::MAX - 2), true);
    assert_eq!(bitmap.contains(u32::MAX - 1), false);
    assert_eq!(bitmap.contains(u32::MAX), true);
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

#[cfg(test)]
fn cardinality_round(data: Vec<u32>) -> bool {
    let original = Bitmap::of(&data);
    let mut a = data.clone();
    a.sort();
    a.dedup();
    a.len() == original.cardinality() as usize
}

#[test]
fn cardinality_roundtrip() {
    QuickCheck::new()
        .gen(StdGen::new(rand::thread_rng(), 1_00_000))
        .tests(10)
        .max_tests(10)
        .quickcheck(cardinality_round as fn(_) -> _)
}

fn serialization_round_trip(original: Vec<u32>) -> bool {
    let original = Bitmap::of(&original);

    let buffer = original.serialize();

    let deserialized = Bitmap::deserialize(&buffer);

    original == deserialized
}

#[test]
fn serialization_roundtrip() {
    QuickCheck::new()
        .gen(StdGen::new(rand::thread_rng(), 1_00_000))
        .tests(10)
        .max_tests(10)
        .quickcheck(serialization_round_trip as fn(_) -> _)
}

#[test]
fn test_serialization_roundtrip() {
    let original: Bitmap = (1..1000).collect();

    let buffer = original.serialize();

    let deserialized = Bitmap::deserialize(&buffer);

    assert_eq!(buffer.len(), original.get_serialized_size_in_bytes());
    assert_eq!(original, deserialized);
}
