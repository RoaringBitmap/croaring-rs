use croaring::Bitmap;
use roaring::RoaringBitmap;

use criterion::measurement::Measurement;
use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkGroup, BenchmarkId, Criterion,
};

fn compare<Prep1, Prep2, Bench1, Bench2, In1, In2, Out1, Out2, M>(
    group: &mut BenchmarkGroup<'_, M>,
    mut prep1: Prep1,
    mut prep2: Prep2,
    mut bench1: Bench1,
    mut bench2: Bench2,
) where
    Prep1: FnMut() -> In1,
    Prep2: FnMut() -> In2,
    Bench1: FnMut(In1) -> Out1,
    Bench2: FnMut(In2) -> Out2,
    M: Measurement,
{
    group.bench_function("croaring", |b| {
        b.iter_batched(|| prep1(), |x| bench1(x), BatchSize::SmallInput);
    });
    group.bench_function("roaring-rs", |b| {
        b.iter_batched(|| prep2(), |x| bench2(x), BatchSize::SmallInput);
    });
}

fn roaring_bitmap_of(items: &[u32]) -> RoaringBitmap {
    items.iter().copied().collect()
}

fn create(c: &mut Criterion) {
    compare(
        &mut c.benchmark_group("create"),
        || (),
        || (),
        |()| Bitmap::create(),
        |()| RoaringBitmap::new(),
    );
}

fn create_and_add_one(c: &mut Criterion) {
    compare(
        &mut c.benchmark_group("add_one"),
        || (),
        || (),
        |()| {
            let mut bitmap = Bitmap::create();
            bitmap.add(black_box(1));
            bitmap
        },
        |()| {
            let mut bitmap = RoaringBitmap::new();
            bitmap.insert(black_box(1));
            bitmap
        },
    );
}

const SIMPLE_ITEMS: &[u32] = &[1, 10, 100, 1_000, 10_000, 100_000, 1_000_000];

fn add(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_several");
    compare(
        &mut group,
        Bitmap::create,
        RoaringBitmap::new,
        |mut bitmap: Bitmap| {
            for &item in SIMPLE_ITEMS {
                bitmap.add(black_box(item));
            }
            bitmap
        },
        |mut bitmap: RoaringBitmap| {
            for &item in SIMPLE_ITEMS {
                bitmap.insert(black_box(item));
            }
            bitmap
        },
    );
    group.bench_function("croaring many", |b| {
        b.iter_batched(
            Bitmap::create,
            |mut bitmap| {
                bitmap.add_many(black_box(SIMPLE_ITEMS));
                bitmap
            },
            BatchSize::SmallInput,
        );
    });
}

fn remove(c: &mut Criterion) {
    compare(
        &mut c.benchmark_group("remove"),
        || Bitmap::of(SIMPLE_ITEMS),
        || roaring_bitmap_of(SIMPLE_ITEMS),
        |mut bitmap: Bitmap| bitmap.remove(black_box(1_000_000)),
        |mut bitmap: RoaringBitmap| bitmap.remove(black_box(1_000_000)),
    );
}

fn contains(c: &mut Criterion) {
    compare(
        &mut c.benchmark_group("contains"),
        || Bitmap::of(&[1]),
        || roaring_bitmap_of(&[1]),
        |bitmap: Bitmap| bitmap.contains(black_box(1)),
        |bitmap: RoaringBitmap| bitmap.contains(black_box(1)),
    )
}

fn cardinality(c: &mut Criterion) {
    let mut group = c.benchmark_group("cardinality");

    for &size in &[100_000, 1_000_000] {
        group.bench_with_input(BenchmarkId::new("croaring", size), &size, |b, &size| {
            b.iter_batched(
                || Bitmap::from_iter(1..size),
                |bitmap: Bitmap| bitmap.cardinality(),
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("roaring-rs", size), &size, |b, &size| {
            b.iter_batched(
                || RoaringBitmap::from_iter(1..size),
                |bitmap: RoaringBitmap| bitmap.len(),
                BatchSize::SmallInput,
            );
        });
    }
}

fn binops(c: &mut Criterion) {
    let range1 = 1..100;
    let range2 = 100..200;

    let gen_ours = || {
        (
            Bitmap::from_iter(range1.clone()),
            Bitmap::from_iter(range2.clone()),
        )
    };
    let gen_theirs = || {
        (
            RoaringBitmap::from_iter(range1.clone()),
            RoaringBitmap::from_iter(range2.clone()),
        )
    };

    macro_rules! comp_op {
        ($new1:ident, $inplace1:ident, $new2:expr, $inplace2:expr $(,)?) => {{
            compare(
                &mut c.benchmark_group(concat!(stringify!($new1), "_new")),
                gen_ours,
                gen_theirs,
                |(bm1, bm2)| bm1.$new1(&bm2),
                |(bm1, bm2)| $new2(&bm1, &bm2),
            );
            compare(
                &mut c.benchmark_group(concat!(stringify!($new1), "_inplace")),
                gen_ours,
                gen_theirs,
                |(mut bm1, bm2)| bm1.$inplace1(&bm2),
                |(mut bm1, bm2)| $inplace2(&mut bm1, &bm2),
            );
        }};
    }

    comp_op!(
        and,
        and_inplace,
        std::ops::BitAnd::bitand,
        std::ops::BitAndAssign::bitand_assign,
    );
    comp_op!(
        or,
        or_inplace,
        std::ops::BitOr::bitor,
        std::ops::BitOrAssign::bitor_assign,
    );
    comp_op!(
        xor,
        xor_inplace,
        std::ops::BitXor::bitxor,
        std::ops::BitXorAssign::bitxor_assign,
    );
}

fn iter(c: &mut Criterion) {
    compare(
        &mut c.benchmark_group("iter"),
        || Bitmap::from_iter(1..10_000),
        || RoaringBitmap::from_iter(1..10_000),
        |bitmap: Bitmap| assert_eq!(bitmap.iter().fold(0, |a, b| a + b), 49995000),
        |bitmap: RoaringBitmap| assert_eq!(bitmap.iter().fold(0, |a, b| a + b), 49995000),
    );
}

criterion_group!(
    benches,
    create,
    create_and_add_one,
    add,
    remove,
    contains,
    cardinality,
    binops,
    iter
);
criterion_main!(benches);
