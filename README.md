# mdpdf

Markdown-to-PDF transducer with built-in unicode math support.

Converts markdown to beautifully typeset PDFs using embedded typst. Handles unicode math, Greek letters, and special characters natively — zero external dependencies.

## Install

```bash
# From source
cargo install --path .

# Via nix flake
nix profile install github:flowerornament/mdpdf
```

## Usage

```bash
# Single file
mdpdf doc.md                        # → doc.pdf

# Explicit output
mdpdf doc.md -o output.pdf

# Multiple files (parallel)
mdpdf *.md

# From stdin
cat doc.md | mdpdf -o doc.pdf

# Batch with structured output
mdpdf *.md --json

# Preview generated typst source
mdpdf doc.md --dry-run

# Custom layout
mdpdf doc.md --margin 0.75in --font-size 12pt --no-toc

# Nushell pipeline
ls *.md | get name | each { mdpdf $in --json } | from json
```

## Features

- **Native unicode** — Greek letters, math operators, arrows, set theory, sub/superscripts, blackboard bold — all rendered natively by typst
- **LaTeX math** — `$$E = mc^2$$` fenced math blocks rendered via mitex
- **Zero dependencies** — typst compiler and fonts are embedded in the binary
- **Parallel rendering** — batch converts up to 8 files concurrently (configurable with `-J`)
- **Structured output** — `--json` emits one JSONL object per file with timing and error details
- **Dry run** — `--dry-run` prints the generated typst source without rendering

## Flags

| Flag | Default | Description |
|------|---------|-------------|
| `-o, --output` | `<stem>.pdf` | Output file path |
| `--toc / --no-toc` | `--toc` | Table of contents |
| `--number-sections / --no-number-sections` | `--number-sections` | Section numbering |
| `--margin` | `1in` | Page margins |
| `--font-size` | `11pt` | Font size |
| `--include-preamble` | — | Additional typst code to prepend |
| `-j, --json` | off | JSONL structured output |
| `--dry-run` | off | Print typst source only |
| `-J, --jobs` | `8` | Max parallel render jobs |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All files rendered successfully |
| 1 | One or more render failures |

## License

MIT
