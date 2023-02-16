#![no_main]

use bitvec::prelude::*;
use croaring::Bitmap;
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use std::mem;
use std::ops::RangeInclusive;

fuzz_target!(|input: FuzzInput| {
    let mut lhs = Bitmap::create();
    let mut rhs = Bitmap::create();

    let mut lhs_check = bitvec![0; 4 * 0x1_0000];
    let mut rhs_check = bitvec![0; 4 * 0x1_0000];

    for op in input.lhs_ops {
        op.on_roaring(&mut lhs);
        op.on_bitvec(&mut lhs_check);
        check_equal(&lhs, &lhs_check);
    }
    for op in input.rhs_ops {
        op.on_roaring(&mut rhs);
        op.on_bitvec(&mut rhs_check);
        check_equal(&rhs, &rhs_check);
    }

    for op in &input.comp_ops {
        op.on_roaring(&mut lhs, &rhs);
        op.on_bitvec(&mut lhs_check, &rhs);
        check_equal(&lhs, &lhs_check);
    }

    for op in &input.view_ops {
        op.on_roaring(&rhs, &rhs_check);
        op.on_roaring(&lhs, &lhs_check);
    }
    /*
    let mut v = Vec::new();
    for side in [lhs.clone(), rhs] {
        v.clear();
        let data = side.serialize();
        let data2 = side.serialize_frozen_into(&mut v);
        let view1 = unsafe { BitmapView::deserialize(&data) };
        assert_eq!(view1, side);
        let view2 = unsafe { BitmapView::deserialize_frozen(data2) };
        assert_eq!(view2, side);
        for op in &input.view_ops {
            op.on_roaring(&view1);
            op.on_roaring(&view2);
        }
        for op in &input.comp_ops {
            op.on_roaring(&mut lhs, &view1);
            op.on_roaring(&mut lhs, &view2);
        }
    }
     */
});

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    lhs_ops: Vec<MutableBitmapOperation>,
    rhs_ops: Vec<MutableBitmapOperation>,
    comp_ops: Vec<BitmapCompOperation>,
    view_ops: Vec<ReadBitmapOp>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
struct Num(u32);

const MAX_NUM: u32 = 0x1_0000 * 4;

impl<'a> Arbitrary<'a> for Num {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(u.int_in_range(0..=(MAX_NUM - 1))?))
    }
}

#[derive(Arbitrary, Debug)]
enum MutableBitmapOperation {
    Add(Num),
    AddChecked(Num),
    AddMany(Vec<Num>),
    AddRange(RangeInclusive<Num>),
    RemoveRange(RangeInclusive<Num>),
    Clear,
    Remove(Num),
    RemoveChecked(Num),
    FlipInplace(RangeInclusive<Num>),
    ShrinkToFit,
    RunOptimize,
    RemoveRunCompression,
    // Probably turn it into a bitmap
    MakeBitmap { key: u16 },
    MakeRange { key: u16 },
}

#[derive(Arbitrary, Debug)]
enum ReadBitmapOp {
    ContainsRange(RangeInclusive<Num>),
    Contains(Num),
    RangeCardinality(RangeInclusive<Num>),
    Cardinality,
    Flip(RangeInclusive<Num>),
    ToVec,
    GetSerializedSizeInBytes,
    GetFrozenSerializedSizeInBytes,
    Serialize,
    SerializeFrozen,
    IsEmpty,
    AddOffset(i64),
    IntersectWithRange(RangeInclusive<Num>),
    Minimum,
    Maximum,
    Rank(Num),
    Select(Num),
    Statistics,
    Clone,
    Debug,
    WithIter(Vec<IterOperation>),
}

#[derive(Arbitrary, Debug)]
enum BitmapCompOperation {
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

#[derive(Arbitrary, Debug)]
enum IterOperation {
    ResetAtOrAfter(u32),
    ReadNext,
    NextMany(u16),
}

impl ReadBitmapOp {
    fn on_roaring(&self, b: &Bitmap, v: &BitSlice) {
        match *self {
            ReadBitmapOp::ContainsRange(ref r) => {
                assert_eq!(
                    v[r.start().0 as usize..=r.end().0 as usize].all(),
                    b.contains_range(r.start().0..=r.end().0)
                );
            }
            ReadBitmapOp::Contains(i) => {
                assert_eq!(v[i.0 as usize], b.contains(i.0));
            }
            ReadBitmapOp::RangeCardinality(ref r) => {
                assert_eq!(
                    v[r.start().0 as usize..=r.end().0 as usize].count_ones() as u64,
                    b.range_cardinality(r.start().0..=r.end().0)
                );
            }
            ReadBitmapOp::Cardinality => {
                assert_eq!(v.count_ones() as u64, b.cardinality());
            }
            ReadBitmapOp::Flip(ref r) => {
                b.flip(r.start().0..=r.end().0);
            }
            ReadBitmapOp::ToVec => {
                let vec_iter = b.to_vec();
                assert!(vec_iter.into_iter().eq(v.iter_ones().map(|i| i as u32)));
            }
            ReadBitmapOp::GetSerializedSizeInBytes => {
                b.get_serialized_size_in_bytes();
            }
            ReadBitmapOp::GetFrozenSerializedSizeInBytes => {
                b.get_frozen_serialized_size_in_bytes();
            }
            ReadBitmapOp::Serialize => {
                b.serialize();
            }
            ReadBitmapOp::SerializeFrozen => {
                let mut v = Vec::new();
                b.serialize_frozen_into(&mut v);
            }
            ReadBitmapOp::IsEmpty => {
                assert_eq!(v.not_any(), b.is_empty());
            }
            ReadBitmapOp::IntersectWithRange(ref r) => {
                assert_eq!(
                    v[r.start().0 as usize..=r.end().0 as usize].any(),
                    b.intersect_with_range(r.start().0..=r.end().0)
                );
            }
            ReadBitmapOp::Minimum => {
                assert_eq!(v.first_one().map(|i| i as u32), b.minimum());
            }
            ReadBitmapOp::Maximum => {
                assert_eq!(v.last_one().map(|i| i as u32), b.maximum());
            }
            ReadBitmapOp::Rank(i) => {
                assert_eq!(
                    v.iter_ones().take_while(|&n| n <= i.0 as usize).count() as u64,
                    b.rank(i.0),
                );
            }
            ReadBitmapOp::Select(i) => {
                assert_eq!(
                    v.iter_ones().nth(i.0 as usize).map(|n| n as u32),
                    b.select(i.0),
                );
            }
            ReadBitmapOp::Statistics => {
                b.statistics();
            }
            ReadBitmapOp::Clone => {
                drop(b.clone());
            }
            ReadBitmapOp::Debug => {
                use std::io::Write;
                write!(std::io::sink(), "{:?}", b).unwrap();
            }
            ReadBitmapOp::WithIter(ref iter_ops) => {
                let mut iter = b.iter();
                for op in iter_ops {
                    match *op {
                        IterOperation::ResetAtOrAfter(i) => {
                            iter.reset_at_or_after(i);
                        }
                        IterOperation::ReadNext => {
                            iter.next();
                        }
                        IterOperation::NextMany(n) => {
                            let mut v = vec![0; n as usize];
                            iter.next_many(&mut v);
                        }
                    }
                }
            }
            ReadBitmapOp::AddOffset(i) => {
                b.add_offset(i);
            }
        }
    }
}

impl MutableBitmapOperation {
    fn on_roaring(&self, b: &mut Bitmap) {
        match *self {
            MutableBitmapOperation::Add(i) => {
                b.add(i.0);
            }
            MutableBitmapOperation::AddChecked(i) => {
                b.add_checked(i.0);
            }
            MutableBitmapOperation::AddMany(ref items) => {
                let items: &[u32] = unsafe { mem::transmute(&items[..]) };
                b.add_many(&items);
            }
            MutableBitmapOperation::AddRange(ref r) => {
                b.add_range(r.start().0..=r.end().0);
            }
            MutableBitmapOperation::RemoveRange(ref r) => {
                b.remove_range(r.start().0..=r.end().0);
            }
            MutableBitmapOperation::Clear => {
                b.clear();
            }
            MutableBitmapOperation::Remove(i) => {
                b.remove(i.0);
            }
            MutableBitmapOperation::RemoveChecked(i) => {
                b.remove_checked(i.0);
            }
            MutableBitmapOperation::FlipInplace(ref r) => {
                b.flip_inplace(r.start().0..=r.end().0);
            }
            MutableBitmapOperation::ShrinkToFit => {
                b.shrink_to_fit();
            }
            MutableBitmapOperation::RunOptimize => {
                b.run_optimize();
            }
            MutableBitmapOperation::RemoveRunCompression => {
                b.remove_run_compression();
            }
            MutableBitmapOperation::MakeBitmap { key } => {
                let key = u32::from(key);
                let start = key * 0x1_0000;
                let end = start + 9 * 1024;
                for i in (start..end).step_by(2) {
                    b.add(i);
                }
            }
            MutableBitmapOperation::MakeRange { key } => {
                let key = u32::from(key);
                let start = key * 0x1_0000;
                let end = start + 0x0_FFFF;
                b.add_range(start..=end)
            }
        }
        b.remove_range(MAX_NUM..);
    }

    fn on_bitvec(&self, b: &mut BitSlice) {
        match *self {
            MutableBitmapOperation::Add(i) | MutableBitmapOperation::AddChecked(i) => {
                b.set(i.0 as usize, true);
            }
            MutableBitmapOperation::AddMany(ref items) => {
                for i in items {
                    b.set(i.0 as usize, true);
                }
            }
            MutableBitmapOperation::AddRange(ref r) => {
                b[r.start().0 as usize..=r.end().0 as usize].fill(true);
            }
            MutableBitmapOperation::RemoveRange(ref r) => {
                b[r.start().0 as usize..=r.end().0 as usize].fill(false);
            }
            MutableBitmapOperation::Clear => {
                b.fill(false);
            }
            MutableBitmapOperation::Remove(i) | MutableBitmapOperation::RemoveChecked(i) => {
                b.set(i.0 as usize, false);
            }
            MutableBitmapOperation::FlipInplace(ref r) => {
                let _ = !&mut b[r.start().0 as usize..=r.end().0 as usize];
            }
            MutableBitmapOperation::ShrinkToFit
            | MutableBitmapOperation::RunOptimize
            | MutableBitmapOperation::RemoveRunCompression => {}
            MutableBitmapOperation::MakeBitmap { key } => {
                if key < (MAX_NUM / 0x1_0000) as u16 {
                    let key = usize::from(key);
                    let start = key * 0x1_0000;
                    let end = start + 9 * 1024;
                    for i in (start..end).step_by(2) {
                        b.set(i, true);
                    }
                }
            }
            MutableBitmapOperation::MakeRange { key } => {
                if key < (MAX_NUM / 0x1_0000) as u16 {
                    let key = usize::from(key);
                    let start = key * 0x1_0000;
                    let end = start + 0x0_FFFF;
                    b[start..=end].fill(true);
                }
            }
        }
    }
}

impl BitmapCompOperation {
    fn on_roaring(&self, lhs: &mut Bitmap, rhs: &Bitmap) {
        match *self {
            BitmapCompOperation::Eq => {
                drop(lhs == rhs);
                assert_eq!(lhs, lhs);
            }
            BitmapCompOperation::IsSubset => {
                lhs.is_subset(rhs);
                assert!(lhs.is_subset(lhs));
            }
            BitmapCompOperation::IsStrictSubset => {
                lhs.is_strict_subset(rhs);
                assert!(!lhs.is_strict_subset(lhs));
            }
            BitmapCompOperation::Intersect => {
                lhs.intersect(rhs);
                assert!(lhs.is_empty() || lhs.intersect(lhs));
            }
            BitmapCompOperation::JacardIndex => {
                lhs.jaccard_index(rhs);
                lhs.jaccard_index(lhs);
            }
            BitmapCompOperation::And => {
                assert_eq!(lhs.and(lhs), *lhs);

                let res = lhs.and(rhs);
                assert_eq!(res.cardinality(), lhs.and_cardinality(rhs));
                lhs.and_inplace(rhs);
                assert_eq!(*lhs, res);
            }
            BitmapCompOperation::Or => {
                assert_eq!(lhs.or(lhs), *lhs);

                let res = lhs.or(rhs);
                assert_eq!(res.cardinality(), lhs.or_cardinality(rhs));
                assert_eq!(res, Bitmap::fast_or(&[lhs, rhs]));
                assert_eq!(res, Bitmap::fast_or_heap(&[lhs, rhs]));

                lhs.or_inplace(rhs);
                assert_eq!(*lhs, res);
            }
            BitmapCompOperation::Xor => {
                assert!(lhs.xor(lhs).is_empty());

                let res = lhs.xor(rhs);
                assert_eq!(res.cardinality(), lhs.xor_cardinality(rhs));
                assert_eq!(res, Bitmap::fast_xor(&[lhs, rhs]));

                lhs.xor_inplace(rhs);
                assert_eq!(*lhs, res);
            }
            BitmapCompOperation::AndNot => {
                assert!(lhs.andnot(lhs).is_empty());

                let res = lhs.andnot(rhs);
                assert_eq!(res.cardinality(), lhs.andnot_cardinality(rhs));

                lhs.andnot_inplace(rhs);
                assert_eq!(*lhs, res);
            }
        }
    }

    fn on_bitvec(&self, lhs: &mut BitSlice, rhs: &Bitmap) {
        match *self {
            BitmapCompOperation::Eq
            | BitmapCompOperation::IsSubset
            | BitmapCompOperation::IsStrictSubset
            | BitmapCompOperation::Intersect
            | BitmapCompOperation::JacardIndex => {}
            BitmapCompOperation::And => {
                let tmp = to_bitvec(rhs, lhs.len());
                *lhs &= &tmp;
            }
            BitmapCompOperation::Or => {
                for i in rhs.iter() {
                    let i = i as usize;
                    if i >= lhs.len() {
                        break;
                    }
                    lhs.set(i, true);
                }
            }
            BitmapCompOperation::Xor => {
                for i in rhs.iter() {
                    let i = i as usize;
                    if i >= lhs.len() {
                        break;
                    }
                    let old_val = *lhs.get(i).unwrap();
                    lhs.set(i, !old_val);
                }
            }
            BitmapCompOperation::AndNot => {
                for i in rhs.iter() {
                    let i = i as usize;
                    if i >= lhs.len() {
                        break;
                    }
                    lhs.set(i, false);
                }
            }
        }
    }
}

fn to_bitvec(b: &Bitmap, max: usize) -> BitVec {
    let mut res = bitvec![0; max];
    for i in b.iter() {
        let i = i as usize;
        if i >= max {
            break;
        }
        res.set(i, true);
    }
    res
}

fn check_equal(b: &Bitmap, v: &BitSlice) {
    let lhs = b.iter().take_while(|&i| i < v.len() as u32);
    let rhs = v.iter_ones().map(|i| i as u32);

    assert!(lhs.eq(rhs), "{b:?}")
}
