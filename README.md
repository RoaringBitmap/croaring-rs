# croaring-rs [![https://travis-ci.org/RoaringBitmap/croaring-rs](https://travis-ci.org/RoaringBitmap/croaring-rs.svg?branch=master)](https://travis-ci.org/RoaringBitmap/croaring-rs)
A [Rust](https://www.rust-lang.org) wrapper for CRoaring (a C/C++ implementation at https://github.com/RoaringBitmap/CRoaring)

The original java version can be found at https://github.com/RoaringBitmap/RoaringBitmap

### Bitmap usage example

```rust
use croaring::Bitmap;

let mut rb1 = Bitmap::new();
rb1.add(1);
rb1.add(2);
rb1.add(3);
rb1.add(4);
rb1.add(5);
rb1.add(100);
rb1.add(1000);
rb1.run_optimize();

let mut rb2 = Bitmap::new();
rb2.add(3);
rb2.add(4);
rb2.add(1000);
rb2.run_optimize();

let mut rb3 = Bitmap::new();

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

println!("{:?}", rb3.to_vec());
println!("{:?}", rb3);
println!("{:?}", rb4);

rb4 = Bitmap::fast_or(&[&rb1, &rb2, &rb3]);

println!("{:?}", rb4);
```

For 64bit Bitmap support, checkout the [`Treemap`](https://docs.rs/croaring/0.4.0/croaring/treemap/struct.Treemap.html). `Treemap` is not API-compatible with `Bitmap`, yet most the functionality is overlapping.

### Treemap usage example

```rust
use std::u64;
use croaring::Treemap;

let mut treemap = Treemap::new();
treemap.add(u64::MAX);
treemap.remove(u64::MAX);

/// Serialization compatible with croaring Treemap version at https://github.com/RoaringBitmap/CRoaring/blob/b88b002407b42fafaea23ea5009a54a24d1c1ed4/cpp/roaring64map.hh

use croaring::treemap::NativeSerializer;

let mut treemap1 = Treemap::new();

for i in 100..1000 {
  treemap1.add(i);
}

treemap1.add(std::u32::MAX as u64);
treemap1.add(std::u64::MAX);

/// Serialization compatible with JVM Treemap version at https://github.com/RoaringBitmap/RoaringBitmap/blob/34654b2d5c3e75e7f9bca1672f4c0b5800d60cf3/roaringbitmap/src/main/java/org/roaringbitmap/longlong/Roaring64NavigableMap.java
use croaring::treemap::JvmSerializer;

let mut treemap2 = Treemap::new();

for i in 100..1000 {
  treemap2.add(i);
}

treemap2.add(std::u32::MAX as u64);
treemap2.add(std::u64::MAX);
```

### Building

```
git clone --recursive https://github.com/RoaringBitmap/croaring-rs/
cd croaring-rs
cargo build
```

In `croaring-rs`, just like in [CRoaring](https://github.com/RoaringBitmap/CRoaring/),
some CPU related code optimizations are enabled dynamically at runtime. If you are
building binaries for specific CPU architectures you can specify `ROARING_ARCH` environment 
variable to control enabled code optimizations, e.g. 
`ROARING_ARCH=ivybridge cargo build --release`.

### Testing

Running unit tests and doc tests:

```
cargo test
```

Running benchmark suite (currently on Rust nightly toolchain only):

```
cargo bench
```

### Documentation

Current documentation is available at https://docs.rs/croaring/latest/croaring/

## CRoaring Version

This crate uses [CRoaring version `4.2.3`](https://github.com/RoaringBitmap/CRoaring/releases/tag/v4.2.3).
The version of this crate does not necessarily match the version of CRoaring: the major version of the crate is only
incremented when there are breaking changes in the Rust API: It is possible (and has happened) that breaking changes
in the CRoaring C API do not necessitate a major version bump in this crate.