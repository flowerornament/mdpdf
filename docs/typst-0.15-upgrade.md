# Typst 0.15 Upgrade & Feature Plan

Status: brainstorm / proposal. Not yet scheduled.

This document captures what Typst 0.15.0 makes possible for mdpdf, what an
upgrade actually involves, and a sequenced plan to land the work in
independently-shippable pieces.

## TL;DR

- The headline obstacle is **not** Typst API churn — it's that
  `typst-as-lib` (our font-embedding and package-resolution glue) has no
  release targeting Typst 0.15 yet. Resolving that is the load-bearing
  decision.
- The three wins worth pursuing, in priority order: **(1)** the dependency
  bump itself (free math/font quality), **(2)** `--pdf-standard` for tagged
  / PDF-A / PDF-UA output, **(3)** an `--html` backend that exports math as
  MathML.

## 1. The dependency blocker

Today's pipeline leans on `typst-as-lib` for three things: the
`TypstEngine` builder, typst-kit font options (embedded fonts), and the
`FileResolver` trait we implement in `src/render.rs`. As of the latest
release (`typst-as-lib` 0.15.5, May 2026) the crate still pins
`typst = "^0.14"`. Its own `0.15.x` versioning is unrelated to Typst's
release numbers.

So we cannot simply bump `typst`/`typst-pdf` to 0.15 — Cargo will not
resolve a single `typst` version across our direct deps and `typst-as-lib`.

### Dependency reality

| Crate | Current | For Typst 0.15 |
|---|---|---|
| `typst`, `typst-pdf` | 0.14 | 0.15 — trivial bump |
| `typst-as-lib` | 0.15 (pins `typst ^0.14`) | **no compatible release yet** |
| `cmarker` | 0.1.8 (min Typst 0.14) | likely fine; verify on 0.15 |
| `mitex` | 0.2.6 | re-verify on 0.15 |
| `typst-embedded-package` | 0.4 | re-verify |

### Options to unblock

1. **Wait** for `typst-as-lib` to ship a Typst-0.15 build. Lowest effort,
   unknown timeline. Track upstream issues/branches before assuming this.
2. **Drop `typst-as-lib`** and implement the `typst::World` trait
   directly. It only provides the engine builder, typst-kit font options,
   and the file resolver — all replaceable. Estimated ~150–250 lines: a
   `World` impl that serves the embedded `main.typ`, resolves embedded
   package files from `PACKAGE_DATA`, and wires fonts via `typst-kit`
   directly (we already depend on its concepts transitively). This removes
   a layer and unblocks us regardless of upstream cadence.
3. **Fork/patch** `typst-as-lib` to bump its `typst` pin. Fastest to a
   green build, but adds a maintenance burden (patch section in
   `Cargo.toml`, or a vendored fork).

Recommendation: check upstream first (option 1). If there's no imminent
0.15 release, prototype option 2 — owning the `World` impl is a small,
well-understood surface and frees us from the wrapper's release cadence
permanently.

## 2. What 0.15 buys this tool

Filtered to "markdown → PDF for LLM technical content." Most of the
changelog (multiple bibliographies, `typst eval`, `--pretty`, backslash
path breakage) is irrelevant to an embedded, filesystem-free transducer.

### Win A — PDF tagging + multiple PDF standards (highest value)

0.15 emits tagged PDFs and can target PDF/A and PDF/UA, several at once.
For a tool that already has clean structural markdown (headings, lists,
tables), this is nearly-free semantic structure for accessibility and
archival compliance — exactly what downstream consumers of generated
reports ask for.

- Surface: `--pdf-standard <a-2b|a-3b|ua-1|...>` (repeatable).
- Implementation touches `PdfOptions` in `compile_to_pdf`
  (`src/render.rs`), currently `PdfOptions::default()`.
- Squarely on-mission; small flag surface.

### Win B — Free quality on upgrade

Refined cramped styles, delimiter and accent rendering, smarter paragraph
grouping, and New Computer Modern 8.1.0 (updated calligraphic
letterforms — and NCM is our default math/text font). LLM math output
renders slightly better with zero new flags. This ships automatically with
the bump.

### Win C — HTML export with MathML (expansion play, approved for pursuit)

0.15 exports equations to MathML out of the box. Because the pipeline
already produces real Typst math via mitex, `mdpdf doc.md --html` could
emit standalone HTML where math is copyable, accessible, and
web-embeddable — same input, second backend.

Open questions to resolve during design:

- Does `cmarker` render cleanly into Typst's HTML export target? cmarker
  emits Typst content; HTML export viability depends on the elements it
  produces having HTML representations. Needs a spike.
- Does mitex math survive into MathML via the HTML target, or does it need
  the native `math.equation` path? This determines whether the HTML
  backend can reuse the current template unchanged.
- Output shape: single self-contained `.html` (inline CSS) vs. assets.
- Routing: `--html` changes the default output extension to `.html`;
  interaction with `-o`, batch mode, and `--json` needs to mirror the
  existing PDF rules in SPEC.md.

This widens the tool's remit. It earns its own design pass and PR; it is
explicitly *not* bundled into the upgrade PR.

### Out of scope (for now)

`--bleed` / spot colors (print production, wrong audience), `--png` /
`--svg` preview export (cheap if ever wanted — SVGs are minified by
default in 0.15), variable-font internals.

## 3. Sequenced plan

Three independently-shippable steps, smallest first. Each is its own PR.

### Step 1 — Unblock + bump (load-bearing)

1. Investigate `typst-as-lib` upstream for Typst-0.15 support; decide
   wait vs. own-`World` vs. fork.
2. Move `typst`/`typst-pdf` to 0.15; align `typst-as-lib` (or its
   replacement) and re-verify `cmarker`, `mitex`,
   `typst-embedded-package`.
3. Re-render the test corpus; confirm math/tables/unicode still correct
   and capture the NCM 8.1.0 visual delta.
4. `just check` + `just build` green. No user-facing flags change.

Breaking-change audit (low risk given our minimal template): `lr` size
param now relative to base glyph size; `class` applies only to direct
body; deprecated symbol/function removals. Our `template.typ` is small and
uses stable primitives, so exposure is minimal but should be eyeballed.

### Step 2 — `--pdf-standard`

1. Add the flag in `src/cli.rs` (validated enum, repeatable).
2. Thread into `PdfOptions` in `compile_to_pdf`.
3. Document in README.md, SPEC.md (flags table), CHANGELOG.md.
4. Tests: flag parsing + a render asserting standard-conformant output.

### Step 3 — `--html` backend (own design pass)

Gated on the open questions above. Likely a new export path alongside the
PDF one in `src/render.rs`, with output routing mirroring the PDF rules.
Worth a dedicated proposal before coding.

## 4. Verification checklist for the upgrade PR

- [ ] Single `typst` version resolves across the dependency graph.
- [ ] `cmarker` + `mitex` render unchanged on 0.15 (math, tables, unicode).
- [ ] Test corpus re-rendered; visual diffs reviewed (NCM 8.1.0).
- [ ] `just check`, `just build`, `nix build .` green.
- [ ] No unintended flag/behavior changes (Step 1 is a pure bump).
