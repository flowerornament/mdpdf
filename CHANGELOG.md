# Changelog

All notable changes to `mdpdf` are documented in this file.

## v0.2.0 - 2026-05-29

### Changed

- Ships `mdpdf` as a standalone embedded-typst markdown-to-PDF transducer with native Unicode rendering, LaTeX math via mitex, embedded fonts/packages, and no external rendering CLI dependencies.
- Supports file, stdin, and parallel batch conversion with structured JSONL output, dry-run Typst generation, layout flags, and clear render-error reporting.
- Includes Nix flake packaging and the local release flow that publishes `origin/release` for downstream flake consumers.
