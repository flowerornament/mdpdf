# mdpdf Specification

## Origin

mdpdf crystallizes a 181-line `scripts/build-pdfs.sh` from the murail project into a standalone, general-purpose UNIX tool.

### Original pandoc invocation

```bash
pandoc "$md" -o "$out" \
  --pdf-engine=tectonic \
  -V geometry:margin=1in -V fontsize=11pt -V documentclass=article \
  --toc --number-sections \
  --include-in-header="$HEADER"
```

### Original LaTeX preamble

The header included these packages and settings:
- `fontspec` — OpenType font support under XeTeX/LuaTeX
- `newunicodechar` — maps bare unicode codepoints to LaTeX commands
- `amsmath, amssymb` — standard math symbols
- `stmaryrd` — additional math operators (semantic brackets, etc.)
- `etoolbox` — hooks for environment modification

Layout tolerances:
- `\tolerance=2000` — allow looser line breaking to avoid overfull hboxes
- `\emergencystretch=5em` — last resort stretch before giving up on line breaking
- `\hfuzz=2pt` — suppress warnings for lines overflowing by less than 2pt
- `\tabcolsep=3pt` — tighter table columns for wide specification tables
- `\AtBeginEnvironment{longtable}{\footnotesize}` — shrink long tables to fit

80+ `\newunicodechar` mappings covering Greek letters, math operators, arrows, set theory, subscripts/superscripts, and blackboard bold.

### What worked

- pandoc + tectonic as the rendering pipeline (reliable, no TeX Live dependency hell)
- The unicode character mappings (LLM output renders correctly)
- Layout tolerances (wide tables and dense content handled gracefully)
- Parallel execution via xargs (fast batch rendering)

### What didn't

- `2>/dev/null` swallowed all pandoc/tectonic errors — failures were silent
- Sequential collection of files, parallel rendering was bolted on
- Temp file cleanup via shell traps — fragile, race-prone
- Project-specific file discovery logic (hardcoded paths, exclusion patterns)
- No structured output — just `ok`/`FAIL` text, no machine-readable results

## Design

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
| 2 | Missing required dependency (pandoc or tectonic not on PATH) |

### Flags

| Flag | Default | Rationale |
|------|---------|-----------|
| `--toc` | on | Technical docs benefit from TOC |
| `--number-sections` | on | Cross-referencing needs section numbers |
| `--margin 1in` | `1in` | Standard readable margin |
| `--font-size 11pt` | `11pt` | Slightly larger than default for readability |
| `--document-class article` | `article` | Most common for technical documents |
| `--include-header FILE` | none | Escape hatch for custom LaTeX |
| `--json` / `-j` | off | Machine-readable output |
| `--dry-run` | off | Debug without rendering |
| `--jobs N` / `-J N` | 8 | Match typical core count |

## Robustness

### Dependency detection

At startup, check for `pandoc` and `tectonic` on PATH using the `which` crate. Exit with code 2 and a clear error message if either is missing. Skip this check in `--dry-run` mode.

### Error reporting

Capture pandoc/tectonic stderr and relay it in the `error` field of `RenderResult`. Never swallow errors. In human mode, show the first 10 lines of error output indented under the FAIL line.

### Temp file cleanup

The LaTeX preamble is written to a `NamedTempFile` (from the `tempfile` crate). The file is automatically deleted when the `NamedTempFile` is dropped — RAII cleanup, no signal handler needed.

### Parallel rendering

Uses rayon's thread pool, not shell xargs. Each render job gets its own temp file for the preamble. Thread count configurable via `--jobs`.

### Edge cases

- **Empty input**: pandoc handles gracefully (produces minimal PDF)
- **Binary input**: pandoc will error, captured and reported
- **Missing files**: checked by pandoc, error captured
- **Very large files**: streaming to disk via pandoc — no memory issues in mdpdf itself
- **Unicode gaps**: preamble audited against common LLM output patterns; extended from 80 to 150+ mappings

### Unicode coverage

Extended beyond the original script to cover:
- Complete Greek alphabet (lowercase + uppercase)
- Full subscript/superscript digit ranges (₀-₉, ⁰-⁹)
- Additional math operators (≡, ∘, √, ∝, ≪, ≫, ≺, ≻)
- Category theory arrows (↪, ↠, ⟶, ⟵)
- Logic symbols (⊢, ⊣, ⊨, ⊩)
- Lattice operators (⊑, ⊒)
- Calligraphic letters (ℒ, ℋ, ℱ, 𝒪)
- Additional blackboard bold (ℚ, ℂ, 𝔽)
- Delimiters (⟨, ⟩, ⌊, ⌋, ⌈, ⌉)
- Calculus (∇, ∫, ∮)
- Miscellaneous (…, ⋯, ⋮, ⋱, ℓ, ℏ, ℘, ℵ)
