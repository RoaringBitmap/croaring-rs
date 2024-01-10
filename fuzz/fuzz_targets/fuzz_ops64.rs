#![no_main]

use crate::arbitrary_ops64::*;
use croaring::{Bitmap64, Portable, Treemap};
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

mod arbitrary_ops64;

fuzz_target!(|input: FuzzInput| {
    let mut lhs64 = Bitmap64::deserialize::<Portable>(input.initial_input);
    let mut rhs64 = Bitmap64::new();
    let mut lhs_tree = Treemap::from_iter(lhs64.iter());
    let mut rhs_tree = Treemap::new();

    for op in input.lhs_ops.iter().take(10) {
        op.on_bitmap64(&mut lhs64);
        op.on_treemap(&mut lhs_tree);
    }
    for op in input.rhs_ops.iter().take(10) {
        op.on_bitmap64(&mut rhs64, &lhs64);
        op.on_treemap(&mut rhs_tree, &lhs_tree);
    }
    for op in input.compares.iter().take(10) {
        op.compare_with_tree(&lhs64, &rhs64, &lhs_tree, &rhs_tree);
    }
    for op in input.view_ops.iter().take(10) {
        op.check_against_tree(&lhs64, &lhs_tree);
    }

    assert_64_eq(&lhs64, &lhs_tree);
    assert_64_eq(&rhs64, &rhs_tree);
});

#[derive(Arbitrary, Debug)]
struct FuzzInput<'a> {
    lhs_ops: Vec<MutableBitmapOperation>,
    rhs_ops: Vec<MutableRhsBitmapOperation>,
    compares: Vec<BitmapCompOperation>,
    view_ops: Vec<ReadBitmapOp>,
    initial_input: &'a [u8],
}
