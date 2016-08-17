extern crate ctest;

fn main() {
    let mut cfg = ctest::TestGenerator::new();

    cfg.header("roaring/roaring.h");
    cfg.include("../croaring-sys/CRoaring/include");
    cfg.generate("../croaring-sys/src/lib.rs", "all.rs");
}
