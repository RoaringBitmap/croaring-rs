#![no_main]

use croaring::{Bitmap, BitmapView};
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use std::mem;
use std::ops::RangeInclusive;

fuzz_target!(|input: FuzzInput| {
    let mut lhs = Bitmap::create();
    let mut rhs = Bitmap::create();

    for op in input.lhs_ops {
        op.do_it(&mut lhs);
    }
    for op in input.rhs_ops {
        op.do_it(&mut rhs);
    }

    for op in &input.comp_ops {
        op.do_it(&mut rhs, &lhs);
    }

    for op in &input.view_ops {
        op.do_it(&rhs);
        op.do_it(&lhs);
    }
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
            op.do_it(&view1);
            op.do_it(&view2);
        }
        for op in &input.comp_ops {
            op.do_it(&mut lhs, &view1);
            op.do_it(&mut lhs, &view2);
        }
    }
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

impl<'a> Arbitrary<'a> for Num {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(u.arbitrary::<u32>()? % (0x1_0000 * 4)))
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
    SetEveryOther { key: u16 },
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
    fn do_it(&self, b: &Bitmap) {
        match *self {
            ReadBitmapOp::ContainsRange(ref r) => {
                b.contains_range(r.start().0..=r.end().0);
            }
            ReadBitmapOp::Contains(i) => {
                b.contains(i.0);
            }
            ReadBitmapOp::RangeCardinality(ref r) => {
                b.range_cardinality(r.start().0..=r.end().0);
            }
            ReadBitmapOp::Cardinality => {
                b.cardinality();
            }
            ReadBitmapOp::Flip(ref r) => {
                b.flip(r.start().0..=r.end().0);
            }
            ReadBitmapOp::ToVec => {
                b.to_vec();
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
                b.is_empty();
            }
            ReadBitmapOp::IntersectWithRange(ref r) => {
                b.intersect_with_range(r.start().0..=r.end().0);
            }
            ReadBitmapOp::Minimum => {
                b.minimum();
            }
            ReadBitmapOp::Maximum => {
                b.maximum();
            }
            ReadBitmapOp::Rank(i) => {
                b.rank(i.0);
            }
            ReadBitmapOp::Select(i) => {
                b.select(i.0);
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
    fn do_it(self, b: &mut Bitmap) {
        match self {
            MutableBitmapOperation::Add(i) => {
                b.add(i.0);
            }
            MutableBitmapOperation::AddChecked(i) => {
                b.add_checked(i.0);
            }
            MutableBitmapOperation::AddMany(items) => {
                let items: &[u32] = unsafe { mem::transmute(&items[..]) };
                b.add_many(&items);
            }
            MutableBitmapOperation::AddRange(r) => {
                b.add_range(r.start().0..=r.end().0);
            }
            MutableBitmapOperation::RemoveRange(r) => {
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
            MutableBitmapOperation::FlipInplace(r) => {
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
            MutableBitmapOperation::SetEveryOther { key } => {
                let key = u64::from(key);
                for i in (key * 0x1_0000..=(key + 1) * 0x1_0000).step_by(2) {
                    b.add(i as u32);
                }
            }
        }
    }
}

impl BitmapCompOperation {
    fn do_it(&self, lhs: &mut Bitmap, rhs: &Bitmap) {
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
}
