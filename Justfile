all: build test

build:
    cargo build

build-release:
    cargo build --release --verbose

test:
    cargo test --workspace

test-release:
    cargo test --workspace --release --verbose

coverage:
    cargo tarpaulin --verbose --workspace -o Html

build-time:
    cargo +nightly clean
    cargo +nightly build -Z timings

fmt:
    cargo fmt --check

lint:
    cargo clippy -- -D warnings
