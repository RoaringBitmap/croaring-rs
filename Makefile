# https://stackoverflow.com/questions/18136918/how-to-get-current-relative-directory-of-your-makefile
working_directory = $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
croaring_source = $(working_directory)/croaring-sys/CRoaring

release:
	cargo build --release

lint:
	cargo fmt --all -- --check
	cargo clippy

test:
	cargo test

croaring_release_url_base = https://github.com/RoaringBitmap/CRoaring/releases/download

# Automatically fetch and update to the latest CRoaring release from GitHub
update_latest_croaring:
	$(eval LATEST_VERSION := $(shell curl -s "https://api.github.com/repos/RoaringBitmap/CRoaring/releases/latest" | jq -r '.tag_name' | sed 's/^v//'))
	@if [ -z "$(LATEST_VERSION)" ]; then \
		echo "Error: Could not fetch latest CRoaring version"; \
		exit 1; \
	fi
	@echo "Latest CRoaring version: $(LATEST_VERSION)"
	@$(MAKE) update_croaring version=$(LATEST_VERSION)

# Fetch the c source amalgamation from a tagged CRoaring release (like `make update_croaring version=0.9.3`)
update_croaring: download_croaring bindgen update_readme_croaring_version update_croaring_sys_version
	@cargo test
	@cd fuzz && cargo test

download_croaring:
	rm -f '$(croaring_source)/roaring.c' '$(croaring_source)/roaring.h' '$(croaring_source)/roaring.hh'
	curl -L --output-dir '$(croaring_source)' \
		-O '$(croaring_release_url_base)/v$(version)/roaring.c' \
		-O '$(croaring_release_url_base)/v$(version)/roaring.h' \
		-O '$(croaring_release_url_base)/v$(version)/roaring.hh'

# Regenerate bindgen bindings
bindgen:
	cd '$(croaring_source)' && \
		bindgen --generate-inline-functions \
			--allowlist-item '(?i-u:roaring|bitset).*' \
			--allowlist-var '(?i-u:roaring|bitset).*' \
			--no-layout-tests \
			--rust-target 1.70 \
			--use-core \
			-o bindgen_bundled_version.rs \
			roaring.h


# sed -i is a GNU extension, so we use a temporary file explicitly
update_readme_croaring_version:
	@echo "Updating README.md with CRoaring version $(version)"
	@sed -r -e 's_\[CRoaring version `[0-9]+\.[0-9]+\.[0-9]+`\]\([^\)]+\)_[CRoaring version `$(version)`](https://github.com/RoaringBitmap/CRoaring/releases/tag/v$(version))_' README.md > README.md.tmp
	@mv README.md.tmp README.md

# We don't always want to update the version of croaring-sys dependency in croaring, but we always want to update croaring-sys
update_croaring_sys_version:
	@echo "Updating croaring-sys version in Cargo.toml to $(version)"
	@sed -r -e 's/^version = ".*"/version = "$(version)"/' croaring-sys/Cargo.toml > croaring-sys/Cargo.toml.tmp
	@mv croaring-sys/Cargo.toml.tmp croaring-sys/Cargo.toml

# Build a c program to (re)generate the example serialized files for testing
test_serialization_files:
	cd croaring/tests/data/ && \
		cc create_serialization.c '$(croaring_source)/roaring.c' -I '$(croaring_source)' -Wall -o create_serialization && \
		./create_serialization

_get_cargo_fuzz:
	command -v cargo-fuzz >/dev/null 2>&1 || cargo install cargo-fuzz

fuzz: _get_cargo_fuzz
	cd fuzz && \
		cargo fuzz check && \
		ASAN_OPTIONS="detect_leaks=1 detect_stack_use_after_return=1" \
		CC=clang CFLAGS=-fsanitize=address \
		cargo fuzz run fuzz_ops -s address -- -max_len=10000 -rss_limit_mb=4096
