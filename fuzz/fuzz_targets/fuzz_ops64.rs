#![no_main]

use crate::arbitrary_ops64::*;
use croaring::{Bitmap64, Portable, Treemap};
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use std::collections::HashSet;

mod arbitrary_ops64;

fuzz_target!(|input: FuzzInput| {
    let mut lhs64 = Bitmap64::deserialize::<Portable>(input.initial_input);
    let mut rhs64 = Bitmap64::new();
    let mut lhs_tree = Treemap::from_iter(lhs64.iter());
    let mut rhs_tree = Treemap::new();

    for op in input.lhs_ops.iter() {
        op.on_bitmap64(&mut lhs64);
        lhs64.internal_validate().unwrap();
        op.on_treemap(&mut lhs_tree);
    }
    for op in input.rhs_ops.iter() {
        op.on_bitmap64(&mut rhs64, &lhs64);
        rhs64.internal_validate().unwrap();
        op.on_treemap(&mut rhs_tree, &lhs_tree);
    }
    for op in input.compares.iter() {
        op.compare_with_tree(&lhs64, &rhs64, &lhs_tree, &rhs_tree);
    }
    for op in input.view_ops.iter() {
        op.check_against_tree(&lhs64, &lhs_tree);
    }

    assert_64_eq(&lhs64, &lhs_tree);
    assert_64_eq(&rhs64, &rhs_tree);
});

#[derive(Arbitrary, Debug)]
struct FuzzInput<'a> {
    lhs_ops: Vec<MutableBitmapOperation>,
    rhs_ops: Vec<MutableRhsBitmapOperation>,
    compares: HashSet<BitmapCompOperation>,
    view_ops: HashSet<ReadBitmapOp>,
    initial_input: &'a [u8],
}
