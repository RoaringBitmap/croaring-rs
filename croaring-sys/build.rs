extern crate bindgen;
extern crate gcc;

use std::env;
use std::path::PathBuf;

fn main() {
    let mut config = gcc::Config::new();

    config.flag("-std=c11");
    config.flag("-march=native");
    config.flag("-O3");
    config.file("CRoaring/roaring.c");
    config.compile("libroaring.a");

    let bindings = bindgen::Builder::default()
        .no_unstable_rust()
        .header("CRoaring/roaring.h")
        .generate_inline_functions(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("croaring-sys.rs"))
        .expect("Couldn't write bindings!");
}
