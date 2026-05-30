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
just --list           # show grouped workflows
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

### Justfile Conventions

- `just` and `just --list` are the discoverable command surface. Keep recipe comments and `[group(...)]` attributes current instead of maintaining a separate hand-written help recipe.
- Release recipes use `just` attributes for safety: `[arg(...)]` validates semver arguments, `quote()` escapes shell interpolation, and `[confirm(...)]` guards the tagging step.

## Release Flow

Release automation is local-first and tag-driven. Day-to-day work lands on `main`; release commits are ordinary commits on `main`; downstream flake consumers that want the latest published release should track `refs/heads/release`.

The `release` branch is generated state. `just release-tag` moves it to the new annotated version tag with `--force-with-lease` after the tag push.

Before bumping, verify shipped behavior is reflected in the docs agents and users read:

- `CHANGELOG.md` — entry for the target version, scaffolded by `release-bump`
- `README.md` — install instructions, command examples, and user-facing behavior
- `SPEC.md` — behavior-contract changes, when command semantics changed
- `AGENTS.md` — release/process changes, when agent workflow changed

Write docs as if they were always correct, without "added" or "updated" language.

Canonical sequence:

```bash
just release-bump 0.2.1
# Fill CHANGELOG.md and update docs for shipped user-facing behavior.
git add Cargo.toml Cargo.lock flake.nix CHANGELOG.md README.md SPEC.md AGENTS.md
git commit -m "Release v0.2.1"
just release-verify
git push origin main
just release-tag 0.2.1
git ls-remote origin refs/heads/release 'refs/tags/v0.2.1^{}'
```

`just release-verify` intentionally requires a clean worktree. Commit the release-prep changes before running it so the Nix build sees the same git-tracked source that will be tagged.

`just release-verify` checks version alignment across `Cargo.toml`, `Cargo.lock`, and `flake.nix`; CHANGELOG readiness with no `TODO`/`TBD` placeholders; then runs `just check`, `just build`, `nix build .`, `nix run . -- --help`, and `./target/release/mdpdf --help`.

`just release-tag` creates and pushes `vX.Y.Z`, then publishes `origin/release` at the same commit. It prompts before running because this is the public release step; use `just --yes release-tag X.Y.Z` only for explicit automation. The final `git ls-remote` check should show matching object IDs for `refs/heads/release` and the peeled tag.

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
