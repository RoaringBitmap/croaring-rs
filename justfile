#!/usr/bin/env just --justfile

croaring_source := justfile_directory() / "croaring-sys/CRoaring"

release:
  cargo build --release

lint:
  cargo clippy

test:
  cargo test

# regenerate bindgen bindings
bindgen:
  cd {{croaring_source}} && \
    bindgen --generate-inline-functions \
      --allowlist-function 'roaring.*' \
      --allowlist-type 'roaring.*' \
      --allowlist-var '(?i:roaring).*' \
      -o bindgen_bundled_version.rs \
      roaring.h

test_serialization_files:
  cd croaring/tests/data/ && \
    cc create_serialization.c {{croaring_source / 'roaring.c'}} -I {{croaring_source}} -Wall -o create_serialization && \
    ./create_serialization