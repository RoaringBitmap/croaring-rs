use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=CRoaring");
    println!("cargo:rerun-if-env-changed=ROARING_ARCH");

    let mut build = cc::Build::new();
    let compiler = build.get_compiler();
    build.file("CRoaring/roaring.c");

    // TODO:
    if (env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows") {
        println!("cargo::warning=Disabling AVX");
        build.define("ROARING_DISABLE_AVX", "1");
    }

    if let Ok(target_arch) = env::var("ROARING_ARCH") {
        build.flag_if_supported(&format!("-march={target_arch}"));
    }

    build.flag_if_supported("-Wno-unused-function");
    println!("cargo:warning=compiler {compiler:#?}");
    println!("cargo:warning=build: {build:#?}");
    build.compile("roaring");
}
