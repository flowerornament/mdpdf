# mdpdf

General-purpose markdown-to-PDF transducer with built-in unicode math support.

## Goal

A proper UNIX tool that does one thing well: convert markdown to beautiful PDFs, with good defaults for LLM-generated technical content. No config files — flags only.

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
| `src/render.rs` | Core transducer — pandoc command builder, preamble management |
| `src/report.rs` | `RenderResult` type, human and JSON output formatting |
| `src/preamble.tex` | LaTeX header with 150+ unicode math mappings |

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

## Runtime Dependencies

pandoc and tectonic must be on PATH. They are NOT bundled — the tool checks for them at startup and exits with code 2 if missing.

## Lint Config

```toml
[lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }

[lints.rust]
unsafe_code = "forbid"
```

## Notes

- The preamble is embedded at compile time via `include_str!` — no runtime file lookup
- Temp files use the `tempfile` crate for RAII cleanup (no shell traps needed)
- Parallel rendering via rayon, not shell xargs
- Exit codes: 0 = success, 1 = render failure, 2 = missing dependency
