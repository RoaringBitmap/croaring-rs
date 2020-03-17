extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    let mut builder = cc::Build::new();
    builder
        .flag_if_supported("-std=c11")
        .flag_if_supported("-O3");

    if cfg!(feature = "compat") {
        builder.define("DISABLEAVX", Some("1"));
    }
    else {
        builder.flag_if_supported("-march=native");
    }

    builder
        .file("CRoaring/roaring.c")
        .compile("libroaring.a");

    let bindings = bindgen::Builder::default()
        .blacklist_type("max_align_t")
        .header("CRoaring/roaring.h")
        .generate_inline_functions(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("croaring-sys.rs"))
        .expect("Couldn't write bindings!");
}
