#!/usr/bin/env just --justfile

croaring_source := justfile_directory() / "croaring-sys/CRoaring"

release:
  cargo build --release

lint:
  cargo clippy

test:
  cargo test

# Fetch the tagged version from CRoaring, and update the c source amalgamation
update_croaring version:
  #!/usr/bin/env bash
  set -euxo pipefail

  tmpdir=$(mktemp -d)
  trap "rm -rf '$tmpdir'" EXIT
  cd $tmpdir || exit 1

  git clone --depth=1 --branch 'v{{version}}' "https://github.com/RoaringBitmap/CRoaring.git" .
  ./amalgamation.sh
  cp roaring.{h,hh,c} '{{croaring_source}}'


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