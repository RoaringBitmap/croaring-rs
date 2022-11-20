use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=CRoaring");
    println!("cargo:rerun-if-env-changed=ROARING_ARCH");

    let mut build = cc::Build::new();
    build.file("CRoaring/roaring.c");

    if let Ok(target_arch) = env::var("ROARING_ARCH") {
        build.flag_if_supported(&format!("-march={}", target_arch));
    }

    build.flag_if_supported("-Wno-unused-function");
    build.compile("roaring");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    #[cfg(feature = "buildtime_bindgen")]
    {
        bindgen::Builder::default()
            .header("CRoaring/roaring.h")
            .generate_inline_functions(true)
            .allowlist_function("roaring.*")
            .allowlist_type("roaring.*")
            .allowlist_var("roaring.*")
            .allowlist_var("ROARING.*")
            .generate()
            .unwrap_or_else(|_| panic!("could not run bindgen on header CRoaring/roaring.h"))
            .write_to_file(out_path.join("croaring-sys.rs"))
            .expect("Couldn't write bindings!");
    }
    #[cfg(not(feature = "buildtime_bindgen"))]
    {
        use std::fs;
        fs::copy(
            "CRoaring/bindgen_bundled_version.rs",
            out_path.join("croaring-sys.rs"),
        )
        .expect("Could not copy bindings to output directory");
    }
}
