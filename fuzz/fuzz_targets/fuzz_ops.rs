#![no_main]

use crate::arbitrary_ops::*;
use croaring::{Bitmap, BitmapView};
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

mod arbitrary_ops;

fuzz_target!(|input: FuzzInput| {
    let mut lhs = Bitmap::create();
    let mut rhs = Bitmap::create();

    for op in &input.lhs_ops {
        op.on_roaring(&mut lhs);
    }
    for op in &input.rhs_ops {
        op.on_roaring(&mut rhs);
    }

    for op in &input.comp_ops {
        op.on_roaring(&mut lhs, &rhs);
    }

    for op in &input.view_ops {
        op.on_roaring(&rhs);
        op.on_roaring(&lhs);
    }

    let mut v = Vec::new();
    let to_compare = lhs.clone();
    check_serialized(&mut lhs, &to_compare, &mut v, &input);
    check_serialized(&mut lhs, &rhs, &mut v, &input);
});

fn check_serialized(lhs: &mut Bitmap, to_compare: &Bitmap, v: &mut Vec<u8>, input: &FuzzInput) {
    v.clear();

    let data = to_compare.serialize();
    let data2 = to_compare.serialize_frozen_into(v);
    let view1 = unsafe { BitmapView::deserialize(&data) };
    assert_eq!(view1, *to_compare);
    let view2 = unsafe { BitmapView::deserialize_frozen(data2) };
    assert_eq!(view2, *to_compare);

    for op in &input.view_ops {
        op.on_roaring(&view1);
        op.on_roaring(&view2);
    }
    for op in &input.comp_ops {
        op.on_roaring(lhs, &view1);
        op.on_roaring(lhs, &view2);
    }
}

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    lhs_ops: Vec<MutableBitmapOperation>,
    rhs_ops: Vec<MutableBitmapOperation>,
    comp_ops: Vec<BitmapCompOperation>,
    view_ops: Vec<ReadBitmapOp>,
}

impl ReadBitmapOp {
    pub fn on_roaring(&self, b: &Bitmap) {
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
                drop(b.to_vec());
            }
            ReadBitmapOp::GetSerializedSizeInBytes => {
                b.get_serialized_size_in_bytes();
            }
            ReadBitmapOp::GetFrozenSerializedSizeInBytes => {
                b.get_frozen_serialized_size_in_bytes();
            }
            ReadBitmapOp::IsEmpty => {
                assert_eq!(b.is_empty(), b.cardinality() == 0);
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
                            assert!(iter.next_many(&mut v) <= n as usize);
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
