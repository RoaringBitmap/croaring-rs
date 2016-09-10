#![feature(test)]

extern crate croaring;
extern crate roaring;
extern crate test;

use croaring::Bitmap;
use roaring::RoaringBitmap;
use test::{Bencher, black_box};

#[bench]
fn perf_comp_create_croaring(b: &mut Bencher) {
    b.iter(|| {
        let bitmap = Bitmap::create();

        black_box(bitmap);
    });
}

#[bench]
fn perf_comp_create_rust_roaring(b: &mut Bencher) {
    b.iter(|| {
        let bitmap: RoaringBitmap<u32> = RoaringBitmap::new();

        black_box(bitmap);
    });
}


#[bench]
fn perf_comp_create_and_add_one_croaring(b: &mut Bencher) {
    b.iter(|| {
        let mut bitmap = Bitmap::create();
        bitmap.add(black_box(1));

        black_box(bitmap);
    });
}

#[bench]
fn perf_comp_create_and_add_one_rust_roaring(b: &mut Bencher) {
    b.iter(|| {
        let mut bitmap: RoaringBitmap<u32> = RoaringBitmap::new();
        bitmap.insert(black_box(1));

        black_box(bitmap);
    });
}

#[bench]
fn perf_comp_add_croaring(b: &mut Bencher) {
    let mut bitmap = Bitmap::create();

    b.iter(|| {
        bitmap.add(black_box(1));
        bitmap.add(black_box(10));
        bitmap.add(black_box(100));
        bitmap.add(black_box(1000));
        bitmap.add(black_box(10000));
        bitmap.add(black_box(100000));
        bitmap.add(black_box(1000000));
    });
    black_box(bitmap);
}

#[bench]
fn perf_comp_add_many_croaring(b: &mut Bencher) {
    let mut bitmap = Bitmap::create();

    b.iter(|| {
        let int_slice = &[1, 10, 100, 1000, 10_000, 100_000, 1_000_000];
        bitmap.add_many(int_slice);
    });
    black_box(bitmap);
}

#[bench]
fn perf_comp_add_rust_roaring(b: &mut Bencher) {
    let mut bitmap: RoaringBitmap<u32> = RoaringBitmap::new();

    b.iter(|| {
        bitmap.insert(black_box(1));
        bitmap.insert(black_box(10));
        bitmap.insert(black_box(100));
        bitmap.insert(black_box(1000));
        bitmap.insert(black_box(10000));
        bitmap.insert(black_box(100000));
        bitmap.insert(black_box(1000000));
    });
    black_box(bitmap);
}

#[bench]
fn perf_comp_remove_croaring(b: &mut Bencher) {
    let mut bitmap = Bitmap::create();
    bitmap.add(1);
    bitmap.add(10);
    bitmap.add(100);
    bitmap.add(1000);
    bitmap.add(10000);
    bitmap.add(100000);
    bitmap.add(1000000);

    b.iter(|| {
        bitmap.remove(black_box(1000000));
    });
}

#[bench]
fn perf_comp_remove_rust_roaring(b: &mut Bencher) {
    let mut bitmap: RoaringBitmap<u32> = RoaringBitmap::new();
    bitmap.insert(1);
    bitmap.insert(10);
    bitmap.insert(100);
    bitmap.insert(1000);
    bitmap.insert(10000);
    bitmap.insert(100000);
    bitmap.insert(1000000);

    b.iter(|| {
        bitmap.remove(black_box(1000000));
    });
}

#[bench]
fn perf_comp_contains_true_croaring(b: &mut Bencher) {
    let mut bitmap = Bitmap::create();
    bitmap.add(1);

    b.iter(|| {
        bitmap.contains(black_box(1));
    });
}

#[bench]
fn perf_comp_contains_true_rust_roaring(b: &mut Bencher) {
    let mut bitmap: RoaringBitmap<u32> = RoaringBitmap::new();
    bitmap.insert(1);

    b.iter(|| {
        bitmap.contains(black_box(1));
    });
}

#[bench]
fn perf_comp_contains_false_croaring(b: &mut Bencher) {
    let bitmap = Bitmap::create();

    b.iter(|| {
        bitmap.contains(black_box(1));
    });
}

#[bench]
fn perf_comp_contains_false_rust_roaring(b: &mut Bencher) {
    let bitmap: RoaringBitmap<u32> = RoaringBitmap::new();

    b.iter(|| {
        bitmap.contains(black_box(1));
    });
}

#[bench]
fn perf_comp_cardinality_100000_croaring(b: &mut Bencher) {
    let bitmap: Bitmap = (1..100000).collect();

    b.iter(|| {
        black_box(bitmap.cardinality());
    });
}

#[bench]
fn perf_comp_cardinality_100000_rust_roaring(b: &mut Bencher) {
    let bitmap: RoaringBitmap<u32> = (1..100000).collect();

    b.iter(|| {
        black_box(bitmap.len());
    });
}

#[bench]
fn perf_comp_and_new_croaring(b: &mut Bencher) {
    let bitmap1: Bitmap = (1..100).collect();
    let bitmap2: Bitmap = (100..200).collect();

    b.iter(|| {
        bitmap1.and(black_box(&bitmap2));
    });
}

#[bench]
fn perf_comp_and_new_rust_roaring(b: &mut Bencher) {
    let bitmap1: RoaringBitmap<u32> = (1..100).collect();
    let bitmap2: RoaringBitmap<u32> = (100..200).collect();

    b.iter(|| {
        let bitmap3: RoaringBitmap<u32> = bitmap1.intersection(black_box(&bitmap2)).collect();

        bitmap3
    });
}

#[bench]
fn perf_comp_and_inplace_croaring(b: &mut Bencher) {
    let mut bitmap1: Bitmap = (1..100).collect();
    let bitmap2: Bitmap = (100..200).collect();

    b.iter(|| {
        bitmap1.and_inplace(black_box(&bitmap2));
    });
}

#[bench]
fn perf_comp_and_inplace_rust_roaring(b: &mut Bencher) {
    let mut bitmap1: RoaringBitmap<u32> = (1..100).collect();
    let bitmap2: RoaringBitmap<u32> = (100..200).collect();

    b.iter(|| {
        bitmap1.intersect_with(black_box(&bitmap2));
    });
}

#[bench]
fn perf_comp_or_new_croaring(b: &mut Bencher) {
    let bitmap1: Bitmap = (1..100).collect();
    let bitmap2: Bitmap = (100..200).collect();

    b.iter(|| {
        bitmap1.or(black_box(&bitmap2));
    });
}

#[bench]
fn perf_comp_or_new_rust_roaring(b: &mut Bencher) {
    let bitmap1: RoaringBitmap<u32> = (1..100).collect();
    let bitmap2: RoaringBitmap<u32> = (100..200).collect();

    b.iter(|| {
        let bitmap3: RoaringBitmap<u32> = bitmap1.union(black_box(&bitmap2)).collect();

        bitmap3
    });
}

#[bench]
fn perf_comp_or_inplace_croaring(b: &mut Bencher) {
    let mut bitmap1: Bitmap = (1..100).collect();
    let bitmap2: Bitmap = (100..200).collect();

    b.iter(|| {
        bitmap1.or_inplace(black_box(&bitmap2));
    });
}

#[bench]
fn perf_comp_or_inplace_rust_roaring(b: &mut Bencher) {
    let mut bitmap1: RoaringBitmap<u32> = (1..100).collect();
    let bitmap2: RoaringBitmap<u32> = (100..200).collect();

    b.iter(|| {
        bitmap1.union_with(black_box(&bitmap2));
    });
}

#[bench]
fn perf_comp_xor_new_croaring(b: &mut Bencher) {
    let bitmap1: Bitmap = (1..100).collect();
    let bitmap2: Bitmap = (100..200).collect();

    b.iter(|| {
        bitmap1.xor(black_box(&bitmap2));
    });
}

#[bench]
fn perf_comp_xor_new_rust_roaring(b: &mut Bencher) {
    let bitmap1: RoaringBitmap<u32> = (1..100).collect();
    let bitmap2: RoaringBitmap<u32> = (100..200).collect();

    b.iter(|| {
        let bitmap3: RoaringBitmap<u32> = bitmap1.symmetric_difference(black_box(&bitmap2)).collect();

        bitmap3
    });
}

#[bench]
fn perf_comp_xor_inplace_croaring(b: &mut Bencher) {
    let mut bitmap1: Bitmap = (1..100).collect();
    let bitmap2: Bitmap = (100..200).collect();

    b.iter(|| {
        bitmap1.xor_inplace(black_box(&bitmap2));
    });
}

#[bench]
fn perf_comp_xor_inplace_rust_roaring(b: &mut Bencher) {
    let mut bitmap1: RoaringBitmap<u32> = (1..100).collect();
    let bitmap2: RoaringBitmap<u32> = (100..200).collect();

    b.iter(|| {
        bitmap1.symmetric_difference_with(black_box(&bitmap2));
    });
}

#[bench]
fn perf_comp_iter_croaring(b: &mut Bencher) {
    let bitmap: Bitmap = (1..10000).collect();

    b.iter(|| {
        let mut sum: u32 = 0;

        for (_, element) in bitmap.into_iter().enumerate() {
            sum += *element;
        }

        assert_eq!(sum, 49995000);
    });
}

#[bench]
fn perf_comp_iter_rust_roaring(b: &mut Bencher) {
    let bitmap: RoaringBitmap<u32> = (1..10000).collect();

    b.iter(|| {
        let mut sum: u32 = 0;

        for (_, element) in bitmap.iter().enumerate() {
            sum += element;
        }

        assert_eq!(sum, 49995000);
    });
}
