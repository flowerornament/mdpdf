#import "@preview/cmarker:0.1.8"
#import "@preview/mitex:0.2.6": mitex

#set page(margin: eval(sys.inputs.at("margin", default: "1in")))
#set text(size: eval(sys.inputs.at("font-size", default: "11pt")))

#if sys.inputs.at("number-sections", default: "true") == "true" {
  set heading(numbering: "1.1")
}

#if sys.inputs.at("toc", default: "true") == "true" {
  outline(title: "Contents", depth: 3)
  pagebreak(weak: true)
}

#cmarker.render(sys.inputs.at("content"), math: mitex, smart-punctuation: true)
