extern crate gcc;

fn main() {
    let mut config = gcc::Config::new();

    config.flag("-std=c11");
    config.flag("-march=native");
    config.flag("-O3");
    config.file("CRoaring/roaring.c");
    config.compile("libroaring.a");
}
