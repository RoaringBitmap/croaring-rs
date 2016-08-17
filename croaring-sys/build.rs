extern crate cmake;

fn main() {
    let mut cfg = cmake::Config::new("CRoaring");

    let dst = cfg
        .define("BUILD_STATIC", "ON")
        .build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=roaring");
    println!("cargo:include={}/include", dst.display());
}
