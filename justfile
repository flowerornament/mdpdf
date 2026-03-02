# Format code
fmt:
    cargo fmt

# Run clippy
lint:
    cargo clippy --all-targets

# Run clippy strict (warnings as errors)
lint-strict:
    cargo clippy --all-targets -- -D warnings

# Run tests
test:
    cargo test

# Run all checks (fmt + lint + test)
check:
    cargo fmt --check
    cargo clippy --all-targets
    cargo test

# Release build
build:
    cargo build --release
