# mdpdf

Markdown-to-PDF transducer with built-in unicode math support.

Converts markdown to beautifully typeset PDFs via pandoc + tectonic. Ships with 150+ unicode character mappings that handle the math symbols, Greek letters, and special characters commonly found in LLM-generated technical documents.

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

# Preview the pandoc command
mdpdf doc.md --dry-run

# Custom layout
mdpdf doc.md --margin 0.75in --font-size 12pt --no-toc

# Nushell pipeline
ls *.md | get name | each { mdpdf $in --json } | from json
```

## Features

- **Unicode math** — 150+ character mappings (Greek, operators, arrows, set theory, sub/superscripts, blackboard bold, and more)
- **Layout tolerances** — tuned for wide tables and dense content without overflow warnings
- **Parallel rendering** — batch converts up to 8 files concurrently (configurable with `-J`)
- **Structured output** — `--json` emits one JSONL object per file with timing and error details
- **Dry run** — `--dry-run` prints the pandoc command without executing
- **RAII cleanup** — temp files cleaned up automatically, even on signals

## Flags

| Flag | Default | Description |
|------|---------|-------------|
| `-o, --output` | `<stem>.pdf` | Output file path |
| `--toc / --no-toc` | `--toc` | Table of contents |
| `--number-sections / --no-number-sections` | `--number-sections` | Section numbering |
| `--margin` | `1in` | Page margins |
| `--font-size` | `11pt` | Font size |
| `--document-class` | `article` | LaTeX document class |
| `--include-header` | — | Additional LaTeX header file |
| `-j, --json` | off | JSONL structured output |
| `--dry-run` | off | Print pandoc command only |
| `-J, --jobs` | `8` | Max parallel render jobs |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All files rendered successfully |
| 1 | One or more render failures |
| 2 | Missing dependency (pandoc or tectonic) |

## Requirements

- [pandoc](https://pandoc.org/) — document converter
- [tectonic](https://tectonic-typesetting.github.io/) — LaTeX engine

Both must be on PATH. mdpdf checks for them at startup.

## License

MIT
