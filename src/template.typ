#import "@preview/cmarker:0.1.10"
#import "@preview/mitex:0.2.7": mitex

#set page(margin: eval(sys.inputs.at("margin", default: "1in")))
#set text(font: "New Computer Modern", size: eval(sys.inputs.at("font-size", default: "11pt")))
#set par(justify: true)
#show math.equation: set text(font: "New Computer Modern Math")
#show raw: set text(font: "DejaVu Sans Mono")

// Table columns are sized to their content, so a cell is often only an inch
// or two wide. Justifying that little text opens rivers, so let table cells
// set ragged right and hyphenate instead.
#show table: it => {
  set par(justify: false)
  set text(hyphenate: true)
  it
}

// Identifiers like `Herald.Medium.NodeSchemaCatalog` carry no line-break
// opportunity under Unicode line breaking: a full stop between letters does
// not break, nor does a camel-case hump. Inside a narrow table cell that
// single token overruns its column and collides with the neighbouring one.
// Insert zero-width spaces after separator punctuation and before capitals so
// the token can wrap. Only tokens long enough to overflow are touched, which
// leaves short code spans — the overwhelming majority — byte-for-byte intact
// for copy-paste.
#let zwsp = "\u{200B}"
#let after-group(m) = m.captures.at(0) + zwsp + m.captures.at(1)

#let soften-token(token) = (
  token
    .replace(regex("([^\p{L}\p{N}]+)([\p{L}\p{N}])"), after-group)
    .replace(regex("([\p{Ll}\p{N}])([\p{Lu}])"), after-group)
)

#let add-break-opportunities(source) = source.replace(
  regex("\S{12,}"),
  m => soften-token(m.text),
)

// cmarker emits `#raw(block: false, "…")` for inline code and
// `#raw(block: true, lang: …, "…")` for fenced blocks, both resolved through
// the evaluation scope. Only inline code is softened; fenced blocks keep
// their source verbatim.
#let wrapping-raw(..args) = {
  let named = args.named()
  let positional = args.pos()
  if named.at("block", default: false) or positional.len() != 1 {
    return raw(..args)
  }
  raw(add-break-opportunities(positional.at(0)), ..named)
}

// Defaults below match CLI defaults; build_inputs always provides these keys.
// The set rule stays at the top level: inside an `if` block it would be scoped
// to that block and never reach the document.
#set heading(
  numbering: if sys.inputs.at("number-sections", default: "false") == "true" {
    "1.1"
  } else {
    none
  },
)

#if sys.inputs.at("toc", default: "false") == "true" {
  outline(title: "Contents", depth: 3)
  pagebreak(weak: true)
}

#cmarker.render(
  sys.inputs.at("content"),
  math: mitex,
  smart-punctuation: true,
  scope: (raw: wrapping-raw),
)
