#![no_main]

use crate::arbitrary_ops::*;
use bitvec::prelude::*;
use croaring::{Bitmap, Native, Portable};
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

mod arbitrary_ops;

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
        op.on_both(&rhs, &rhs_check);
        op.on_both(&lhs, &lhs_check);
    }
});

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    lhs_ops: Vec<MutableBitmapOperation>,
    rhs_ops: Vec<MutableBitmapOperation>,
    comp_ops: Vec<BitmapCompOperation>,
    view_ops: Vec<ReadBitmapOp>,
}

impl ReadBitmapOp {
    fn on_both(&self, b: &Bitmap, v: &BitSlice) {
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
            ReadBitmapOp::GetPortableSerializedSizeInBytes => {
                b.get_serialized_size_in_bytes::<Portable>();
            }
            ReadBitmapOp::GetNativeSerializedSizeInBytes => {
                b.get_serialized_size_in_bytes::<Native>();
            }
            ReadBitmapOp::GetFrozenSerializedSizeInBytes => {
                b.get_serialized_size_in_bytes::<Frozen>();
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
            ReadBitmapOp::Index(Num(i)) => {
                let actual = b.position(i);
                if let Some(actual) = actual {
                    assert!(v[i as usize]);
                    assert_eq!(v[..i as usize].count_ones(), actual as usize);
                } else {
                    assert!(!v[i as usize]);
                }
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
