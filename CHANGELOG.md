# Changelog

All notable changes to `mdpdf` are documented in this file.

## v0.3.1 - 2026-08-12

### Changed

- publishes producer-locked native Nix packages to the public flowerornament
  Cachix cache and proves substitution before a release can be tagged
- supports Apple Silicon macOS and aarch64/x86_64 Linux, matching current
  Nixpkgs platform support
- exposes a default flake app and standalone cache-aware Nix installation

## v0.3.0 - 2026-08-04

### Changed

- Wraps long inline-code identifiers inside table cells. Unicode line breaking offers no break inside a dotted or camel-cased identifier, so a token like `Herald.Medium.NodeSchemaCatalog` used to run through its column border and overprint the neighbouring cell. Inline code longer than 12 characters now carries break opportunities after separator punctuation and before capitals; shorter spans and fenced code blocks stay byte-for-byte intact so copied code remains exact.
- Sets table cells ragged right with hyphenation rather than justified, which removes the rivers that justification opened at typical column widths.
- Applies `--number-sections` to the document. The heading numbering was previously set inside a conditional block, which scoped it to that block and left headings unnumbered.
- Renders on typst 0.15 with cmarker 0.1.10 and mitex 0.2.7, alongside refreshed Rust and Nix dependencies.

## v0.2.0 - 2026-05-29

### Changed

- Ships `mdpdf` as a standalone embedded-typst markdown-to-PDF transducer with native Unicode rendering, LaTeX math via mitex, embedded fonts/packages, and no external rendering CLI dependencies.
- Supports file, stdin, and parallel batch conversion with structured JSONL output, dry-run Typst generation, layout flags, and clear render-error reporting.
- Includes Nix flake packaging and the local release flow that publishes `origin/release` for downstream flake consumers.
