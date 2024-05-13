use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput,
};
use std::ops::ControlFlow;

use croaring::{Bitmap, Bitmap64, Portable};

fn new(c: &mut Criterion) {
    c.bench_function("new", |b| b.iter(Bitmap::new));

    c.bench_function("with_capacity", |b| {
        b.iter(|| Bitmap::with_container_capacity(10_000))
    });
}

fn add(c: &mut Criterion) {
    c.bench_function("add", |b| {
        let mut bitmap = Bitmap::new();

        b.iter(|| bitmap.add(10000));
    });
}

fn add_many(c: &mut Criterion) {
    c.bench_function("add_many", |b| {
        let mut bitmap = Bitmap::new();
        let int_slice = &[10, 100, 10_000, 1_000_000, 10_000_000];

        b.iter(|| bitmap.add_many(black_box(int_slice)));
    });
}

fn remove(c: &mut Criterion) {
    c.bench_function("remove", |b| {
        let mut bitmap = Bitmap::new();

        b.iter(|| bitmap.remove(10000));
    });
}

fn contains(c: &mut Criterion) {
    let mut group = c.benchmark_group("contains");
    group.bench_function("true", |b| {
        let mut bitmap = Bitmap::new();

        bitmap.add(5);

        b.iter(|| bitmap.contains(5));
    });

    group.bench_function("false", |b| {
        let bitmap = Bitmap::new();

        b.iter(|| bitmap.contains(5));
    });
}

fn cardinality(c: &mut Criterion) {
    let mut group = c.benchmark_group("cardinality");

    for &size in &[100_000, 1_000_000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let bitmap: Bitmap = (0..size).collect();

            b.iter(|| bitmap.cardinality());
        });
    }
}

fn binops(c: &mut Criterion) {
    let bitmap1 = Bitmap::from([500, 1000]);
    let bitmap2 = Bitmap::from([1000, 2000]);

    macro_rules! bench_op {
        ($new:ident, $inplace:ident) => {{
            let mut group = c.benchmark_group(stringify!($new));

            group.bench_function("new", |b| {
                b.iter(|| bitmap1.$new(&bitmap2));
            });
            group.bench_function("inplace", |b| {
                b.iter_batched(
                    || bitmap1.clone(),
                    |mut dst_bitmap| dst_bitmap.$inplace(&bitmap2),
                    BatchSize::SmallInput,
                );
            });

            group
        }};
        ($new:ident, $inplace:ident, $fast:ident) => {{
            let mut group = bench_op!($new, $inplace);

            group.bench_function("fast", |b| {
                b.iter(|| Bitmap::$fast(&[&bitmap1, &bitmap2]));
            });

            group
        }};
        ($new:ident, $inplace:ident, $fast:ident, $fast_heap:ident) => {{
            let mut group = bench_op!($new, $inplace, $fast);

            group.bench_function("fast_heap", |b| {
                b.iter(|| Bitmap::$fast_heap(&[&bitmap1, &bitmap2]));
            });

            group
        }};
    }

    bench_op!(and, and_inplace);
    bench_op!(or, or_inplace, fast_or, fast_or_heap);
    bench_op!(xor, xor_inplace, fast_xor);
    bench_op!(andnot, andnot_inplace);
}

fn flip(c: &mut Criterion) {
    let bitmap = Bitmap::of(&[1]);

    let mut group = c.benchmark_group("flip");
    group.bench_function("new", |b| {
        b.iter(|| bitmap.flip(1..3));
    });
    group.bench_function("inplace", |b| {
        b.iter_batched(
            || bitmap.clone(),
            |mut bitmap| bitmap.flip_inplace(1..3),
            BatchSize::SmallInput,
        );
    });
}

fn to_vec(c: &mut Criterion) {
    const N: usize = 100_000;
    let bitmap: Bitmap = random_iter().take(N).collect();
    let mut g = c.benchmark_group("collect");
    g.bench_function("to_vec", |b| {
        b.iter(|| bitmap.to_vec());
    });
    g.bench_function("via_iter", |b| {
        b.iter(|| bitmap.iter().collect::<Vec<_>>());
    });
    g.bench_function("foreach", |b| {
        b.iter(|| {
            let mut vec = Vec::with_capacity(bitmap.cardinality() as usize);
            bitmap.for_each(|item| -> ControlFlow<()> {
                vec.push(item);
                ControlFlow::Continue(())
            });
            vec
        });
    });
}

fn get_serialized_size_in_bytes(c: &mut Criterion) {
    c.bench_function("get_serialized_size_in_bytes", |b| {
        let bitmap = Bitmap::of(&[1, 2, 3]);
        b.iter(|| bitmap.get_serialized_size_in_bytes::<Portable>());
    });
}

fn is_empty(c: &mut Criterion) {
    let mut group = c.benchmark_group("is_empty");
    group.bench_function("true", |b| {
        let bitmap = Bitmap::new();
        b.iter(|| bitmap.is_empty());
    });
    group.bench_function("false", |b| {
        let bitmap = Bitmap::of(&[1000]);
        b.iter(|| bitmap.is_empty());
    });
}

fn of(c: &mut Criterion) {
    c.bench_function("of", |b| {
        b.iter(|| Bitmap::of(black_box(&[10, 20, 30, 40])));
    });
}

fn serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize");
    for &size in &[100_000, 1_000_000] {
        let bitmap: Bitmap = (1..size).collect();
        group.throughput(Throughput::Elements(size.into()));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| bitmap.serialize::<Portable>());
        });
    }
}

fn deserialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialize");
    for &size in &[100_000, 1_000_000] {
        let bitmap: Bitmap = (1..size).collect();
        let serialized_buffer = bitmap.serialize::<Portable>();
        group.throughput(Throughput::Elements(size.into()));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| Bitmap::deserialize::<Portable>(&serialized_buffer));
        });
    }
}

fn bulk_new(c: &mut Criterion) {
    const N: u32 = 1_000_000;

    let mut group = c.benchmark_group("bulk_new");
    group.throughput(Throughput::Elements(N.into()));
    let range = black_box(0..N);
    group.bench_function("range_new", |b| {
        b.iter(|| Bitmap::from_range(range.clone()));
    });
    group.bench_function("collect", |b| {
        b.iter(|| Bitmap::from_iter(range.clone()));
    });
    group.bench_function("slice_init", |b| {
        let bulk_data = black_box(range.clone().collect::<Vec<_>>());
        b.iter(|| Bitmap::of(&bulk_data));
    });
    group.bench_function("sequential_adds", |b| {
        b.iter(|| {
            let mut bitmap = Bitmap::new();
            for i in range.clone() {
                bitmap.add(i);
            }
            bitmap
        });
    });

    group.finish();
}

#[derive(Clone)]
struct RandomIter {
    x: u32,
}

impl Iterator for RandomIter {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        const MULTIPLIER: u32 = 742938285;
        const MODULUS: u32 = (1 << 31) - 1;
        self.x = (MULTIPLIER.wrapping_mul(self.x)) % MODULUS;
        Some(self.x)
    }
}

fn random_iter() -> RandomIter {
    RandomIter { x: 20170705 }
}

fn create_random(c: &mut Criterion) {
    const N: u32 = 5_000;
    // Clamp values so we get some re-use of containers
    const MAX: u32 = 8 * (u16::MAX as u32 + 1);

    let mut group = c.benchmark_group("random_iter");
    group.throughput(Throughput::Elements(N.into()));

    let rand_iter = random_iter();

    group.bench_function("random_adds", |b| {
        b.iter(|| {
            let mut bitmap = Bitmap::new();
            rand_iter.clone().take(N as usize).for_each(|item| {
                bitmap.add(item);
            });
            bitmap
        });
    });
    group.bench_function("random_from_iter", |b| {
        b.iter(|| Bitmap::from_iter(rand_iter.clone().take(N as usize)));
    });
    group.bench_function("collect_to_vec_first", |b| {
        b.iter(|| {
            let vec = rand_iter.clone().take(N as usize).collect::<Vec<_>>();
            Bitmap::of(&vec)
        });
    });
}

fn collect_bitmap64_to_vec(c: &mut Criterion) {
    const N: u64 = 1_000_000;

    let mut group = c.benchmark_group("collect_bitmap64_to_vec");
    group.throughput(Throughput::Elements(N.into()));
    let bitmap = Bitmap64::from_range(0..N);
    group.bench_function("to_vec", |b| {
        b.iter_batched(|| (), |()| bitmap.to_vec(), BatchSize::LargeInput);
    });
    group.bench_function("foreach", |b| {
        b.iter_batched(
            || (),
            |()| {
                let mut vec = Vec::with_capacity(bitmap.cardinality() as usize);
                bitmap.for_each(|item| -> ControlFlow<()> {
                    vec.push(item);
                    ControlFlow::Continue(())
                });
                vec
            },
            BatchSize::LargeInput,
        );
    });
    group.bench_function("iter", |b| {
        b.iter_batched(
            || (),
            |()| {
                let mut vec = Vec::with_capacity(bitmap.cardinality() as usize);
                vec.extend(bitmap.iter());
                vec
            },
            BatchSize::LargeInput,
        );
    });
    group.bench_function("iter_many", |b| {
        b.iter_batched(
            || (),
            |()| {
                let mut vec = vec![0; bitmap.cardinality() as usize];
                let mut iter = bitmap.cursor();
                assert_eq!(iter.read_many(&mut vec), vec.len());
                vec
            },
            BatchSize::LargeInput,
        );
    });

    group.finish();
}

fn iterate_bitmap64(c: &mut Criterion) {
    const N: u64 = 1_000_000;
    const END_ITER: u64 = N - 100;

    let mut group = c.benchmark_group("bitmap64_iterate");
    group.throughput(Throughput::Elements(N.into()));
    let bitmap = Bitmap64::from_range(0..N);
    group.bench_function("iter", |b| {
        b.iter(|| {
            for x in bitmap.iter() {
                if x == END_ITER {
                    break;
                }
            }
        })
    });
    group.bench_function("cursor", |b| {
        b.iter(|| {
            let mut cursor = bitmap.cursor();
            while let Some(x) = cursor.next() {
                if x == END_ITER {
                    break;
                }
            }
        })
    });
    group.bench_function("for_each", |b| {
        b.iter(|| {
            bitmap.for_each(|x| -> ControlFlow<()> {
                if x == END_ITER {
                    return ControlFlow::Break(());
                }
                ControlFlow::Continue(())
            })
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    new,
    add,
    add_many,
    remove,
    contains,
    cardinality,
    binops,
    flip,
    to_vec,
    get_serialized_size_in_bytes,
    is_empty,
    of,
    serialize,
    deserialize,
    bulk_new,
    create_random,
    collect_bitmap64_to_vec,
    iterate_bitmap64,
);
criterion_main!(benches);
