use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=CRoaring");
    println!("cargo:rerun-if-env-changed=ROARING_ARCH");

    let mut build = cc::Build::new();
    build.file("CRoaring/roaring.c");

    if let Ok(target_arch) = env::var("ROARING_ARCH") {
        build.flag_if_supported(&format!("-march={target_arch}"));
    }

    build.flag_if_supported("-Wno-unused-function");
    build.compile("roaring");
}
