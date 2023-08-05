use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput,
};

use croaring::{Bitmap, Portable};

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
    let bitmap1 = Bitmap::of(&[500, 1000]);
    let bitmap2 = Bitmap::of(&[1000, 2000]);

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
    c.bench_function("to_vec", |b| {
        let bitmap = Bitmap::of(&[1, 2, 3]);
        b.iter(|| bitmap.to_vec());
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
);
criterion_main!(benches);
