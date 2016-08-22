extern crate ctest;

fn main() {
    let mut cfg = ctest::TestGenerator::new();

    cfg.header("roaring.h");
    cfg.include("../croaring-sys/CRoaring");
    cfg.generate("../croaring-sys/src/lib.rs", "all.rs");
}
