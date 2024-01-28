use croaring::{Bitmap64, Portable, Treemap};
use libfuzzer_sys::arbitrary::{self, Arbitrary, Unstructured};
use std::mem;
use std::ops::RangeInclusive;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Num(pub u64);

pub const MAX_NUM: u64 = 0x1_0000 * 260;

impl<'a> Arbitrary<'a> for Num {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(u.int_in_range(0..=(MAX_NUM - 1))?))
    }
}

#[derive(Arbitrary, Debug, PartialEq, Eq)]
pub enum MutableBitmapOperation {
    Add(Num),
    AddChecked(Num),
    AddMany(Vec<Num>),
    AddRange(RangeInclusive<Num>),
    RemoveRange(RangeInclusive<Num>),
    FlipRange(RangeInclusive<Num>),
    Copy,
    Clear,
    Remove(Num),
    RemoveChecked(Num),
    RunOptimize,
    RemoveRunCompression,
    // Add to the max key (or with 0xFFFFFFFF_FFFF0000)
    AddToMax(u16),
}

#[derive(Arbitrary, Debug, PartialEq, Eq)]
pub enum MutableRhsBitmapOperation {
    MutateSelf(MutableBitmapOperation),
    MutBinaryOp(BitmapMutBinop),
}

#[derive(Arbitrary, Debug, PartialEq, Eq)]
pub enum BitmapMutBinop {
    And,
    Or,
    Xor,
    AndNot,
}

impl BitmapMutBinop {
    fn on_treemap(&self, lhs: &mut Treemap, rhs: &Treemap) {
        match *self {
            BitmapMutBinop::And => {
                lhs.and_inplace(rhs);
            }
            BitmapMutBinop::Or => {
                lhs.or_inplace(rhs);
            }
            BitmapMutBinop::Xor => {
                lhs.xor_inplace(rhs);
            }
            BitmapMutBinop::AndNot => {
                lhs.andnot_inplace(rhs);
            }
        }
    }

    fn on_roaring64(&self, lhs: &mut Bitmap64, rhs: &Bitmap64) {
        match *self {
            BitmapMutBinop::And => {
                let expected = lhs.and(rhs);
                lhs.and_inplace(rhs);
                assert_eq!(expected, *lhs);
            }
            BitmapMutBinop::Or => {
                let expected = lhs.or(rhs);
                lhs.or_inplace(rhs);
                assert_eq!(expected, *lhs);
            }
            BitmapMutBinop::Xor => {
                let expected = lhs.xor(rhs);
                lhs.xor_inplace(rhs);
                assert_eq!(expected, *lhs);
            }
            BitmapMutBinop::AndNot => {
                let expected = lhs.andnot(rhs);
                lhs.andnot_inplace(rhs);
                assert_eq!(expected, *lhs);
            }
        }
    }
}

#[derive(Arbitrary, Debug, PartialEq, Eq)]
pub enum BitmapCompOperation {
    Eq,
    IsSubset,
    IsStrictSubset,
    Intersect,
    JacardIndex,
    And,
    Or,
    Xor,
    AndNot,
}

#[derive(Arbitrary, Debug, PartialEq, Eq)]
pub enum ReadBitmapOp {
    ContainsRange(RangeInclusive<u64>),
    Contains(u64),
    RangeCardinality(RangeInclusive<u64>),
    Cardinality,
    ToVec,
    GetPortableSerializedSizeInBytes,
    PortableSerialize,
    /*
    GetNativeSerializedSizeInBytes,
    GetFrozenSerializedSizeInBytes,
     */
    IsEmpty,
    IntersectWithRange(RangeInclusive<u64>),
    Minimum,
    Maximum,
    Rank(u64),
    Index(u64),
    Select(u64),
    Clone,
    Debug,
}

impl ReadBitmapOp {
    pub fn check_against_tree(&self, b: &Bitmap64, t: &Treemap) {
        match *self {
            ReadBitmapOp::Contains(i) => {
                assert_eq!(b.contains(i), t.contains(i));
            }
            ReadBitmapOp::RangeCardinality(ref r) => {
                // Tree doesn't implement directly, but we can do it manually
                let mut t_with_range = t.clone();
                if !r.is_empty() {
                    t_with_range.remove_range(0..*r.start());
                    t_with_range.remove_range(r.end() + 1..);
                }
                assert_eq!(
                    b.range_cardinality(r.start()..=r.end()),
                    t_with_range.cardinality()
                );
            }
            ReadBitmapOp::Cardinality => {
                assert_eq!(b.cardinality(), t.cardinality());
            }
            ReadBitmapOp::IsEmpty => {
                assert_eq!(b.is_empty(), b.cardinality() == 0);
                assert_eq!(b.is_empty(), t.is_empty());
            }
            ReadBitmapOp::Minimum => {
                assert_eq!(b.minimum(), t.minimum());
            }
            ReadBitmapOp::Maximum => {
                assert_eq!(b.maximum(), t.maximum());
            }
            ReadBitmapOp::Rank(i) => {
                assert_eq!(b.rank(i), t.rank(i));
            }
            ReadBitmapOp::Index(i) => {
                assert_eq!(b.position(i), t.position(i));
            }
            ReadBitmapOp::Select(i) => {
                assert_eq!(b.select(i), t.select(i));
            }
            ReadBitmapOp::Clone => {
                let other = b.clone();
                assert_eq!(b, &other);
            }
            ReadBitmapOp::Debug => {
                use std::io::Write;
                let mut black_hole = std::io::sink();
                write!(black_hole, "{:?}", b).unwrap();
            }
            ReadBitmapOp::ToVec => {
                assert_eq!(b.to_vec(), t.to_vec());
            }
            ReadBitmapOp::GetPortableSerializedSizeInBytes => {
                assert_eq!(
                    b.get_serialized_size_in_bytes::<Portable>(),
                    t.get_serialized_size_in_bytes::<Portable>()
                );
            }
            ReadBitmapOp::PortableSerialize => {
                assert_eq!(b.serialize::<Portable>(), t.serialize::<Portable>(),)
            }
            ReadBitmapOp::ContainsRange(ref range) => {
                // Unsupported by treemaps
                _ = b.contains_range(range.start()..=range.end());
            }
            ReadBitmapOp::IntersectWithRange(ref range) => {
                // Unsupported by treemaps
                _ = b.intersect_with_range(range.start()..=range.end());
            }
        }
    }
}

impl MutableBitmapOperation {
    pub fn on_treemap(&self, t: &mut Treemap) {
        match *self {
            MutableBitmapOperation::Add(Num(i)) => {
                t.add(i);
            }
            MutableBitmapOperation::AddChecked(Num(i)) => {
                let expected = !t.contains(i);
                let result = t.add_checked(i);
                assert_eq!(expected, result);
            }
            MutableBitmapOperation::AddMany(ref items) => {
                for &Num(item) in items {
                    t.add(item)
                }
            }
            MutableBitmapOperation::AddRange(ref r) => {
                t.add_range(r.start().0..=r.end().0);
            }
            MutableBitmapOperation::RemoveRange(ref r) => {
                t.remove_range(r.start().0..=r.end().0);
            }
            MutableBitmapOperation::Clear => {
                t.clear();
            }
            MutableBitmapOperation::Remove(Num(i)) => {
                t.remove(i);
            }
            MutableBitmapOperation::RemoveChecked(Num(i)) => {
                let expected = t.contains(i);
                let result = t.remove_checked(i);
                assert_eq!(expected, result);
            }
            MutableBitmapOperation::RunOptimize => {
                t.run_optimize();
            }
            MutableBitmapOperation::RemoveRunCompression => {
                // TODO: For now, we don't support removing run compression on roaring64, so
                //       we don't do it for treemaps (so we can compare how they serialize)
                // t.remove_run_compression();
            }
            MutableBitmapOperation::Copy => {
                *t = t.clone();
            }
            MutableBitmapOperation::AddToMax(low_bits) => {
                const UPPER_BITS: u64 = 0xFFFF_FFFF_FFFF_0000;
                t.add(UPPER_BITS | u64::from(low_bits));
            }
            MutableBitmapOperation::FlipRange(ref range) => {
                // Tremap's flip is inplace
                () = t.flip(range.start().0..=range.end().0);
            }
        }
    }

    pub fn on_bitmap64(&self, b: &mut Bitmap64) {
        match *self {
            MutableBitmapOperation::Add(Num(i)) => {
                b.add(i);
            }
            MutableBitmapOperation::AddChecked(Num(i)) => {
                let expected = !b.contains(i);
                let result = b.add_checked(i);
                assert_eq!(expected, result);
            }
            MutableBitmapOperation::AddMany(ref items) => {
                let items: &[u64] = unsafe { mem::transmute(&items[..]) };
                b.add_many(items);
            }
            MutableBitmapOperation::AddRange(ref range) => {
                b.add_range(range.start().0..=range.end().0);
            }
            MutableBitmapOperation::RemoveRange(ref range) => {
                b.remove_range(range.start().0..=range.end().0);
            }
            MutableBitmapOperation::Copy => {
                *b = b.clone();
            }
            MutableBitmapOperation::Clear => {
                const UPPER_BITS: u64 = 0xFFFF_FFFF_FFFF_0000;
                if !b.is_empty() {
                    b.remove_range(UPPER_BITS..);
                }
                if !b.is_empty() {
                    b.remove_range(b.minimum().unwrap()..=b.maximum().unwrap())
                }
            }
            MutableBitmapOperation::Remove(Num(i)) => {
                b.remove(i);
            }
            MutableBitmapOperation::RemoveChecked(Num(i)) => {
                let expected = b.contains(i);
                let result = b.remove_checked(i);
                assert_eq!(expected, result);
            }
            MutableBitmapOperation::RunOptimize => {
                b.run_optimize();
            }
            MutableBitmapOperation::RemoveRunCompression => {
                // Unsupported
            }
            MutableBitmapOperation::AddToMax(low_bits) => {
                const UPPER_BITS: u64 = 0xFFFF_FFFF_FFFF_0000;
                b.add(UPPER_BITS | u64::from(low_bits));
            }
            MutableBitmapOperation::FlipRange(ref range) => {
                let expected = b.flip(range.start().0..=range.end().0);
                b.flip_inplace(range.start().0..=range.end().0);
                assert_eq!(expected, *b);
            }
        }
    }
}

impl MutableRhsBitmapOperation {
    pub fn on_treemap(&self, current: &mut Treemap, other: &Treemap) {
        match *self {
            MutableRhsBitmapOperation::MutateSelf(ref op) => {
                op.on_treemap(current);
            }
            MutableRhsBitmapOperation::MutBinaryOp(ref op) => {
                op.on_treemap(current, other);
            }
        }
    }

    pub fn on_bitmap64(&self, current: &mut Bitmap64, other: &Bitmap64) {
        match *self {
            MutableRhsBitmapOperation::MutateSelf(ref op) => {
                op.on_bitmap64(current);
            }
            MutableRhsBitmapOperation::MutBinaryOp(ref op) => {
                op.on_roaring64(current, other);
            }
        }
    }
}

impl BitmapCompOperation {
    pub fn compare_with_tree(
        &self,
        lhs_bitmap: &Bitmap64,
        rhs_bitmap: &Bitmap64,
        lhs_tree: &Treemap,
        rhs_tree: &Treemap,
    ) {
        match *self {
            BitmapCompOperation::Eq => {
                assert_eq!(lhs_bitmap == rhs_bitmap, lhs_tree == rhs_tree);
            }
            BitmapCompOperation::IsSubset => {
                assert_eq!(
                    lhs_bitmap.is_subset(rhs_bitmap),
                    lhs_tree.is_subset(rhs_tree),
                );
            }
            BitmapCompOperation::IsStrictSubset => {
                assert_eq!(
                    lhs_bitmap.is_strict_subset(rhs_bitmap),
                    lhs_tree.is_strict_subset(rhs_tree),
                );
            }
            BitmapCompOperation::Intersect => {
                let tree_intersect = !(lhs_tree & rhs_tree).is_empty();
                assert_eq!(lhs_bitmap.intersect(rhs_bitmap), tree_intersect);
                assert!(lhs_bitmap.is_empty() || lhs_bitmap.intersect(lhs_bitmap));
            }
            BitmapCompOperation::JacardIndex => {
                // Treemap doesn't support jaccard index
                _ = lhs_bitmap.jaccard_index(rhs_bitmap);
                _ = lhs_bitmap.jaccard_index(lhs_bitmap);
            }
            BitmapCompOperation::And => {
                let bitmap = lhs_bitmap & rhs_bitmap;
                let treemap = lhs_tree & rhs_tree;

                assert_64_eq(&bitmap, &treemap);
                assert_eq!(bitmap.cardinality(), lhs_bitmap.and_cardinality(rhs_bitmap));
            }
            BitmapCompOperation::Or => {
                let bitmap = lhs_bitmap | rhs_bitmap;
                let treemap = lhs_tree | rhs_tree;

                assert_64_eq(&bitmap, &treemap);
                assert_eq!(bitmap.cardinality(), lhs_bitmap.or_cardinality(rhs_bitmap));
            }
            BitmapCompOperation::Xor => {
                let bitmap = lhs_bitmap ^ rhs_bitmap;
                let treemap = lhs_tree ^ rhs_tree;

                assert_64_eq(&bitmap, &treemap);
                assert_eq!(bitmap.cardinality(), lhs_bitmap.xor_cardinality(rhs_bitmap));
            }
            BitmapCompOperation::AndNot => {
                let bitmap = lhs_bitmap - rhs_bitmap;
                let treemap = lhs_tree - rhs_tree;

                assert_64_eq(&bitmap, &treemap);
                assert_eq!(
                    bitmap.cardinality(),
                    lhs_bitmap.andnot_cardinality(rhs_bitmap),
                );
            }
        }
    }
}

pub fn assert_64_eq(lhs: &Bitmap64, rhs: &Treemap) {
    assert_eq!(lhs.cardinality(), rhs.cardinality());
    if lhs.serialize::<Portable>() != rhs.serialize::<Portable>() {
        let mut lhs = lhs.iter().enumerate();
        let mut rhs = rhs.iter();
        while let Some((i, l)) = lhs.next() {
            let r = rhs.next().unwrap();
            assert_eq!(l, r, "{l} != {r} at {i}");
        }
        assert!(rhs.next().is_none());
        panic!("Serialize not equal, but all items equal?")
    }
}
