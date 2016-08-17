#![feature(test)]

extern crate croaring;
extern crate test;

use croaring::Bitmap;
use test::Bencher;

#[bench]
fn bench_create(b: &mut Bencher) {
    b.iter(|| {
        let bitmap = Bitmap::create();

        bitmap
    });
}

#[bench]
fn bench_create_with_capacity(b: &mut Bencher) {
    b.iter(|| {
        let bitmap = Bitmap::create_with_capacity(10000);

        bitmap
    });
}

#[bench]
fn bench_add(b: &mut Bencher) {
    let mut bitmap = Bitmap::create();

    b.iter(|| {
        bitmap.add(10000);
    });
}

#[bench]
fn bench_remove(b: &mut Bencher) {
    let mut bitmap = Bitmap::create();

    b.iter(|| {
        bitmap.remove(10000);
    });
}

#[bench]
fn bench_contains_true(b: &mut Bencher) {
    let mut bitmap = Bitmap::create();

    bitmap.add(5);

    b.iter(|| {
        bitmap.contains(5);
    });
}

#[bench]
fn bench_contains_false(b: &mut Bencher) {
    let bitmap = Bitmap::create();

    b.iter(|| {
        bitmap.contains(5);
    });
}

#[bench]
fn bench_cardinality_100000(b: &mut Bencher) {
    let bitmap: Bitmap = (1..100000).collect();

    b.iter(|| {
        bitmap.cardinality();
    });
}

#[bench]
fn bench_cardinality_1000000(b: &mut Bencher) {
    let bitmap: Bitmap = (1..1000000).collect();

    b.iter(|| {
        bitmap.cardinality();
    });
}

#[bench]
fn bench_and(b: &mut Bencher) {
    let mut bitmap1 = Bitmap::create();

    bitmap1.add(500);
    bitmap1.add(1000);

    let mut bitmap2 = Bitmap::create();

    bitmap2.add(1000);
    bitmap2.add(2000);

    b.iter(|| {
        bitmap1.and(&bitmap2);
    });
}

#[bench]
fn bench_and_inplace(b: &mut Bencher) {
    let mut bitmap1 = Bitmap::create();

    bitmap1.add(500);
    bitmap1.add(1000);

    let mut bitmap2 = Bitmap::create();

    bitmap2.add(1000);
    bitmap2.add(2000);

    b.iter(|| {
        bitmap1.and_inplace(&bitmap2);
    });
}

#[bench]
fn bench_or(b: &mut Bencher) {
    let mut bitmap1 = Bitmap::create();

    bitmap1.add(500);
    bitmap1.add(1000);

    let mut bitmap2 = Bitmap::create();

    bitmap2.add(1000);
    bitmap2.add(2000);

    b.iter(|| {
        bitmap1.or(&bitmap2);
    });
}

#[bench]
fn bench_or_inplace(b: &mut Bencher) {
    let mut bitmap1 = Bitmap::create();

    bitmap1.add(500);
    bitmap1.add(1000);

    let mut bitmap2 = Bitmap::create();

    bitmap2.add(1000);
    bitmap2.add(2000);

    b.iter(|| {
        bitmap1.or_inplace(&bitmap2);
    });
}

#[bench]
fn bench_fast_or(b: &mut Bencher) {
    let mut bitmap1 = Bitmap::create();

    bitmap1.add(500);
    bitmap1.add(1000);

    let mut bitmap2 = Bitmap::create();

    bitmap2.add(1000);
    bitmap2.add(2000);

    b.iter(|| {
        Bitmap::fast_or(&[&bitmap1, &bitmap2]);
    });
}

#[bench]
fn bench_fast_or_heap(b: &mut Bencher) {
    let mut bitmap1 = Bitmap::create();

    bitmap1.add(500);
    bitmap1.add(1000);

    let mut bitmap2 = Bitmap::create();

    bitmap2.add(1000);
    bitmap2.add(2000);

    b.iter(|| {
        Bitmap::fast_or_heap(&[&bitmap1, &bitmap2]);
    });
}

#[bench]
fn bench_xor(b: &mut Bencher) {
    let mut bitmap1 = Bitmap::create();

    bitmap1.add(15);
    bitmap1.add(25);

    let mut bitmap2 = Bitmap::create();

    bitmap2.add(25);
    bitmap2.add(35);

    b.iter(|| {
        bitmap1.xor(&bitmap2);
    });
}

#[bench]
fn bench_xor_inplace(b: &mut Bencher) {
    let mut bitmap1 = Bitmap::create();

    bitmap1.add(15);
    bitmap1.add(25);

    let mut bitmap2 = Bitmap::create();

    bitmap2.add(25);
    bitmap2.add(35);

    b.iter(|| {
        bitmap1.xor_inplace(&bitmap2);
    });
}

#[bench]
fn bench_fast_xor(b: &mut Bencher) {
    let mut bitmap1 = Bitmap::create();

    bitmap1.add(15);
    bitmap1.add(25);

    let mut bitmap2 = Bitmap::create();

    bitmap2.add(25);
    bitmap2.add(35);

    b.iter(|| {
        Bitmap::fast_or(&[&bitmap1, &bitmap2]);
    });
}

#[bench]
fn bench_andnot(b: &mut Bencher) {
    let mut bitmap1 = Bitmap::create();

    bitmap1.add(15);
    bitmap1.add(25);

    let mut bitmap2 = Bitmap::create();

    bitmap2.add(25);
    bitmap2.add(35);

    b.iter(|| {
        bitmap1.andnot(&bitmap2);
    });
}

#[bench]
fn bench_andnot_inplace(b: &mut Bencher) {
    let mut bitmap1 = Bitmap::create();

    bitmap1.add(15);
    bitmap1.add(25);

    let mut bitmap2 = Bitmap::create();

    bitmap2.add(25);
    bitmap2.add(35);

    b.iter(|| {
        bitmap1.andnot_inplace(&bitmap2);
    });
}

#[bench]
fn bench_flip(b: &mut Bencher) {
    let mut bitmap = Bitmap::create();

    bitmap.add(1);

    b.iter(|| {
        bitmap.flip((1..3));
    });
}

#[bench]
fn bench_flip_inplace(b: &mut Bencher) {
    let mut bitmap = Bitmap::create();

    bitmap.add(1);

    b.iter(|| {
        bitmap.flip_inplace((1..3));
    });
}

#[bench]
fn bench_as_slice(b: &mut Bencher) {
    let mut bitmap = Bitmap::create();

    bitmap.add(1);
    bitmap.add(2);
    bitmap.add(3);

    b.iter(|| {
        bitmap.as_slice();
    });
}

#[bench]
fn bench_get_serialized_size_in_bytes(b: &mut Bencher) {
    let mut bitmap = Bitmap::create();

    bitmap.add(1);
    bitmap.add(2);
    bitmap.add(3);

    b.iter(|| {
        bitmap.get_serialized_size_in_bytes();
    });
}

#[bench]
fn bench_is_empty_true(b: &mut Bencher) {
    let bitmap = Bitmap::create();

    b.iter(|| {
        bitmap.is_empty();
    });
}

#[bench]
fn bench_is_empty_false(b: &mut Bencher) {
    let mut bitmap = Bitmap::create();

    bitmap.add(1000);

    b.iter(|| {
        bitmap.is_empty();
    });
}

#[bench]
fn bench_of(b: &mut Bencher) {
    b.iter(|| {
        Bitmap::of(&vec![10, 20, 30, 40])
    });
}

#[bench]
fn bench_serialize_100000(b: &mut Bencher) {
    let bitmap: Bitmap = (1..100000).collect();

    b.iter(|| {
        bitmap.serialize();
    });
}

#[bench]
fn bench_serialize_1000000(b: &mut Bencher) {
    let bitmap: Bitmap = (1..1000000).collect();

    b.iter(|| {
        bitmap.serialize();
    });
}

#[bench]
fn bench_deserialize_100000(b: &mut Bencher) {
    let bitmap: Bitmap = (1..100000).collect();
    let serialized_buffer = bitmap.serialize();

    b.iter(|| {
        Bitmap::deserialize(serialized_buffer);
    });
}

#[bench]
fn bench_deserialize_1000000(b: &mut Bencher) {
    let bitmap: Bitmap = (1..1000000).collect();
    let serialized_buffer = bitmap.serialize();

    b.iter(|| {
        Bitmap::deserialize(serialized_buffer);
    });
}
