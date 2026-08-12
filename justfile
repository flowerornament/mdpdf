set shell := ["bash", "-euo", "pipefail", "-c"]
set positional-arguments := false

[private]
default:
    @just --list

# Format code
[group('check')]
fmt:
    cargo fmt

# Run clippy
[group('check')]
lint:
    cargo clippy --all-targets

# Run clippy strict (warnings as errors)
[group('check')]
lint-strict:
    cargo clippy --all-targets -- -D warnings

# Run tests
[group('check')]
test:
    cargo test

# Run all checks (fmt + lint + test)
[group('check')]
check:
    cargo fmt --check
    cargo clippy --all-targets
    cargo test

# Run comparison tests (requires pdftotext)
[group('check')]
test-compare:
    cargo test -- --ignored

# Visual comparison (renders both pipelines, outputs PNGs)
[group('check')]
test-visual:
    bash tests/visual.sh

# Generate pandoc reference PDFs (one-time setup)
[group('fixtures')]
generate-references:
    bash tests/generate_references.sh

# Release build
[group('build')]
build:
    cargo build --release

# Update release versions and scaffold changelog.
[group('release')]
[arg('version', pattern='[0-9]+\.[0-9]+\.[0-9]+', help='Semver release, e.g. 0.2.1')]
release-bump version:
    python3 scripts/release.py bump {{quote(version)}}

# Release-readiness checks and validation.
[group('release')]
release-verify:
    python3 scripts/release.py verify

# Verify all advertised package outputs are in the public cache.
[group('release')]
cache-verify:
    python3 scripts/release.py cache-verify

# Create and push an annotated release tag, then publish origin/release.
[group('release')]
[arg('version', pattern='[0-9]+\.[0-9]+\.[0-9]+', help='Semver release, e.g. 0.2.1')]
[confirm("This will tag and force-update origin/release. Continue?")]
release-tag version:
    python3 scripts/release.py tag {{quote(version)}}
