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

# Run comparison tests (requires pdftotext)
test-compare:
    cargo test -- --ignored

# Visual comparison (renders both pipelines, outputs PNGs)
test-visual:
    bash tests/visual.sh

# Generate pandoc reference PDFs (one-time setup)
generate-references:
    bash tests/generate_references.sh

# Release build
build:
    cargo build --release
