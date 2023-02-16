use croaring::Bitmap;
use libfuzzer_sys::arbitrary::{self, Arbitrary, Unstructured};
use std::mem;
use std::ops::RangeInclusive;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Num(pub u32);

pub const MAX_NUM: u32 = 0x1_0000 * 4;

impl<'a> Arbitrary<'a> for Num {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(u.int_in_range(0..=(MAX_NUM - 1))?))
    }
}

#[derive(Arbitrary, Debug)]
pub enum MutableBitmapOperation {
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
pub enum ReadBitmapOp {
    ContainsRange(RangeInclusive<Num>),
    Contains(Num),
    RangeCardinality(RangeInclusive<Num>),
    Cardinality,
    Flip(RangeInclusive<Num>),
    ToVec,
    GetSerializedSizeInBytes,
    GetFrozenSerializedSizeInBytes,
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

#[derive(Arbitrary, Debug)]
pub enum IterOperation {
    ResetAtOrAfter(u32),
    ReadNext,
    NextMany(u16),
}

impl MutableBitmapOperation {
    pub fn on_roaring(&self, b: &mut Bitmap) {
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
}

impl BitmapCompOperation {
    pub fn on_roaring(&self, lhs: &mut Bitmap, rhs: &Bitmap) {
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
