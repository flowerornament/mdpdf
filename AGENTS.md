# mdpdf

General-purpose markdown-to-PDF transducer with built-in unicode math support.

## Goal

A proper UNIX tool that does one thing well: convert markdown to beautiful PDFs, with good defaults for LLM-generated technical content. No config files — flags only. No external CLI dependencies — typst is embedded.

## Quick Start

```bash
mdpdf doc.md                    # single file → doc.pdf
mdpdf *.md --json               # batch with JSONL output
cat notes.md | mdpdf -o out.pdf # stdin → file
```

## Structure

| File | Purpose |
|------|---------|
| `SPEC.md` | Full specification (origin, design, robustness) |
| `src/main.rs` | Entry point — delegates to lib |
| `src/lib.rs` | `run()` + `run_with()` — orchestration and CLI dispatch |
| `src/cli.rs` | Clap types and flag definitions |
| `src/render.rs` | Core transducer — typst compilation, package resolver, PDF export |
| `src/report.rs` | `RenderResult` type, human and JSON output formatting |
| `src/template.typ` | Typst template with cmarker + mitex integration |
| `src/typst-packages/` | Embedded package tarballs (cmarker, mitex) |

## Build & Quality

This is a **Rust** project. Use `just` for all build operations.

```bash
just check            # fmt + lint + test (the one command you need)
just fmt              # cargo fmt
just lint             # cargo clippy --all-targets
just lint-strict      # clippy with warnings as errors (-D warnings)
just test             # cargo test
just build            # cargo build --release
```

### Toolchain

Managed via `rust-toolchain.toml` — stable channel with rustfmt and clippy components. No manual rustup needed.

### Quality Gate

A Stop hook runs `just check` before ending any session. If fmt, lint, or tests fail, the session blocks until you fix the issues.

## Lint Config

```toml
[lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }

[lints.rust]
unsafe_code = "forbid"
```

## Notes

- The typst template is embedded at compile time via `include_str!` — no runtime file lookup
- Package tarballs (cmarker, mitex) are embedded at compile time via `include_package!`
- Fonts are embedded via `typst-kit` with `embed-fonts` — no system font dependencies
- Parallel rendering via rayon
- Exit codes: 0 = success, 1 = render failure
