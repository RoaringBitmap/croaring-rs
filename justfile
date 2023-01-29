#!/usr/bin/env just --justfile

release:
  cargo build --release    

lint:
  cargo clippy

test:
  cargo test

# regenerate bindgen bindings
bindgen:
  cd croaring-sys/CRoaring && \
    bindgen --generate-inline-functions \
      --allowlist-function 'roaring.*' \
      --allowlist-type 'roaring.*' \
      --allowlist-var '(?i:roaring).*' \
      -o bindgen_bundled_version.rs \
      roaring.h