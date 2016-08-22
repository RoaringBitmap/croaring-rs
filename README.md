# croaring-rs ![https://travis-ci.org/saulius/croaring-rs.svg?branch=master](https://travis-ci.org/saulius/croaring-rs.svg?branch=master)
A [Rust](https://www.rust-lang.org) wrapper for CRoaring (a C/C++ implementation at https://github.com/RoaringBitmap/CRoaring)

The original java version can be found at https://github.com/RoaringBitmap/RoaringBitmap

### Usage example

```rust
use croaring::Bitmap;

let mut rb1 = Bitmap::create();
rb1.add(1);
rb1.add(2);
rb1.add(3);
rb1.add(4);
rb1.add(5);
rb1.add(100);
rb1.add(1000);
rb1.run_optimize();

let mut rb2 = Bitmap::create();
rb2.add(3);
rb2.add(4);
rb2.add(1000);
rb2.run_optimize();

let mut rb3 = Bitmap::create();

assert_eq!(rb1.cardinality(), 7);
assert!(rb1.contains(3));

rb1.and_inplace(&rb2);
rb3.add(5);
rb3.or_inplace(&rb1);

let mut rb4 = Bitmap::fast_or(&[&rb1, &rb2, &rb3]);

rb1.and_inplace(&rb2);
println!("{:?}", rb1);

rb3.add(5);
rb3.or_inplace(&rb1);

println!("{:?}", rb1);

rb3.add(5);
rb3.or_inplace(&rb1);

println!("{:?}", rb3.as_slice());
println!("{:?}", rb3);
println!("{:?}", rb4);

rb4 = Bitmap::fast_or(&[&rb1, &rb2, &rb3]);

println!("{:?}", rb4);
```

### Building locally

```
git checkout https://github.com/saulius/croaring-rs/
cd croaring-rs
git submodule update --init --recursive
cargo build
```

Tested on Rust [stable/beta/nightly and LLVM version 3.8](https://github.com/saulius/croaring-rs/blob/master/.travis.yml).

### Testing

Running unit tests and doc tests:

```
cargo test
```

Running `croaring-sys` sys tests:

```
cargo run --manifest-path systest/Cargo.toml
```

Running benchmark suite (currently on Rust nightly toolchain only):

```
cargo bench
```

### Documentation

Current documentation is available at https://saulius.github.io/croaring-rs/croaring/
