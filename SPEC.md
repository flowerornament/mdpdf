# mdpdf Specification

## Origin

mdpdf crystallizes a 181-line `scripts/build-pdfs.sh` from the murail project into a standalone, general-purpose UNIX tool.

### Original pipeline

The original used pandoc + tectonic with 150+ `\newunicodechar` LaTeX mappings. mdpdf v0.2 replaced this with embedded typst — zero external CLI dependencies, native unicode support.

### What worked

- The unicode character mappings (LLM output renders correctly)
- Layout tolerances (wide tables and dense content handled gracefully)
- Parallel execution (fast batch rendering)

### What didn't

- `2>/dev/null` swallowed all pandoc/tectonic errors — failures were silent
- Sequential collection of files, parallel rendering was bolted on
- Temp file cleanup via shell traps — fragile, race-prone
- Project-specific file discovery logic (hardcoded paths, exclusion patterns)
- No structured output — just `ok`/`FAIL` text, no machine-readable results

## Design

### Architecture

mdpdf compiles markdown to PDF entirely in-process using typst as an embedded library. No external CLI tools are required.

Pipeline: **Markdown → cmarker (CommonMark → typst) → mitex (LaTeX math → typst math) → typst compiler → PDF bytes**

Key crates:
- `typst-as-lib` — Builder wrapping the typst compiler with font embedding
- `typst-pdf` — PDF export from compiled typst documents
- `typst-embedded-package` — Compile-time embedding of typst package tarballs

Embedded packages:
- `@preview/cmarker:0.1.8` — CommonMark rendering in typst
- `@preview/mitex:0.2.6` — LaTeX math rendering in typst

### Transducer model

mdpdf is a UNIX transducer: it takes markdown in and produces PDF out. No state, no config files, no project awareness.

### Input routing

| Condition | Behavior |
|-----------|----------|
| No arguments, stdin is a TTY | Print help and exit |
| No arguments, stdin has data | Read from stdin, require `-o` |
| One file argument | Render file → `<stem>.pdf` (or `-o` path) |
| Multiple file arguments | Render all in parallel, `-o` is an error |

### Output routing

| Flag | Single file | Multiple files |
|------|-------------|----------------|
| (default) | `<stem>.pdf` same directory | Each file → `<stem>.pdf` same directory |
| `-o path` | Render to specified path | Error |

### Structured output (--json)

One JSONL object per file on stdout:

```json
{"input":"doc.md","output":"doc.pdf","success":true,"time_ms":1234}
{"input":"bad.md","output":"bad.pdf","success":false,"time_ms":567,"error":"..."}
```

Human output goes to stderr so stdout stays clean for piping.

### Human output

Progress to stderr:
```
  ok: doc.pdf (1234ms)
  FAIL: bad.md
    Error: ...

Rendered 5 files (1 failures)
```

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | All files rendered successfully |
| 1 | One or more render failures, or invalid arguments |

### Flags

| Flag | Default | Rationale |
|------|---------|-----------|
| `--toc` | on | Technical docs benefit from TOC |
| `--number-sections` | on | Cross-referencing needs section numbers |
| `--margin 1in` | `1in` | Standard readable margin |
| `--font-size 11pt` | `11pt` | Slightly larger than default for readability |
| `--include-preamble FILE` | none | Escape hatch for custom typst code |
| `--json` / `-j` | off | Machine-readable output |
| `--dry-run` | off | Print generated typst source |
| `--jobs N` / `-J N` | 8 | Match typical core count |

## Robustness

### Error reporting

Typst compilation errors are captured and relayed in the `error` field of `RenderResult`. Never swallow errors. In human mode, show the first 10 lines of error output indented under the FAIL line.

### Parallel rendering

Uses rayon's thread pool. Thread count configurable via `--jobs`.

### Unicode coverage

Typst handles unicode natively — no character mapping tables needed. Greek letters, math operators, arrows, subscripts, superscripts, and all other unicode symbols render correctly out of the box.

### Fonts

Fonts are embedded at compile time via `typst-kit` with the `embed-fonts` feature. Default fonts include Libertinus Serif, New Computer Modern, and DejaVu Sans Mono. No system font dependencies at runtime.
