#!/usr/bin/env just --justfile

croaring_source := justfile_directory() / "croaring-sys/CRoaring"

release:
  cargo build --release

lint:
  cargo clippy

test:
  cargo test


croaring_release_url_base := "https://github.com/RoaringBitmap/CRoaring/releases/download"

# Fetch the c source amalgamation from a tagged CRoaring release (like `just update_croaring 0.9.3`)
update_croaring version: && bindgen
  rm -f '{{croaring_source}}/roaring.c' '{{croaring_source}}/roaring.h' '{{croaring_source}}/roaring.hh'
  wget -P '{{croaring_source}}' \
    '{{croaring_release_url_base}}/v{{version}}/roaring.c' \
    '{{croaring_release_url_base}}/v{{version}}/roaring.h' \
    '{{croaring_release_url_base}}/v{{version}}/roaring.hh'

# Regenerate bindgen bindings
bindgen:
  cd {{croaring_source}} && \
    bindgen --generate-inline-functions \
      --allowlist-function 'roaring.*' \
      --allowlist-type 'roaring.*' \
      --allowlist-var '(?i:roaring).*' \
      -o bindgen_bundled_version.rs \
      roaring.h

# Build a c program to (re)generate the example serialized files for testing
test_serialization_files:
  cd croaring/tests/data/ && \
    cc create_serialization.c {{croaring_source / 'roaring.c'}} -I {{croaring_source}} -Wall -o create_serialization && \
    ./create_serialization
