extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    let mut build = cc::Build::new();
    build.file("CRoaring/roaring.c");

    if let Ok(target_arch) = env::var("ROARING_ARCH") {
        build.flag_if_supported(&format!("-march={}", target_arch));
    } else {
        build.flag_if_supported("-march=native");
    }

    build.compile("libroaring.a");

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
