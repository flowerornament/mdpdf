//! Template behaviour tests.
//!
//! These inspect the extracted text layer of a rendered PDF, so they need
//! `pdftotext` on PATH. When it is missing they skip rather than fail, which
//! keeps `just check` usable on a bare machine.

use std::path::Path;
use std::process::Command;

use mdpdf::cli::Cli;
use mdpdf::render::render_one;

/// Zero-width space — the break opportunity the template inserts into long
/// inline-code tokens.
const ZWSP: char = '\u{200B}';

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn has_pdftotext() -> bool {
    Command::new("pdftotext").arg("-v").output().is_ok()
}

/// Render a fixture and return its extracted text, or `None` when `pdftotext`
/// is unavailable.
fn render_text(name: &str, cli: &Cli) -> Option<String> {
    if !has_pdftotext() {
        eprintln!("SKIP: pdftotext not found on PATH");
        return None;
    }

    let dir = tempfile::tempdir().expect("tempdir");
    let output = dir.path().join("out.pdf");
    let result = render_one(&fixture(name), &output, cli);
    assert!(result.success(), "{name}: {:?}", result.error());

    let extracted = Command::new("pdftotext")
        .arg(&output)
        .arg("-")
        .output()
        .expect("pdftotext failed");
    Some(String::from_utf8_lossy(&extracted.stdout).to_string())
}

/// Long dotted identifiers get break opportunities so they can wrap inside a
/// narrow table cell instead of overrunning into the next column.
#[test]
fn long_inline_code_gains_break_opportunities() {
    let Some(text) = render_text("wrapping.md", &Cli::default()) else {
        return;
    };

    // The identifier is long enough that it really does wrap, and extraction
    // splits it wherever the break was taken — so assert on the break
    // opportunities themselves rather than on any particular line layout.
    assert!(
        text.contains(&format!("Herald.{ZWSP}")),
        "separator punctuation should gain a break opportunity"
    );
    assert!(
        text.contains(&format!("Node{ZWSP}")),
        "camel humps should gain a break opportunity"
    );
    assert!(
        !text.contains("Herald.Medium."),
        "unsoftened identifier should not survive — it would overrun its column"
    );
}

/// Short code spans are the overwhelming majority, so they stay byte-for-byte
/// intact and copy-paste cleanly.
#[test]
fn short_inline_code_is_left_alone() {
    let Some(text) = render_text("wrapping.md", &Cli::default()) else {
        return;
    };

    for span in ["foo.bar", "Vec<T>", "a::b", "x = 1"] {
        assert!(
            text.contains(span),
            "short span `{span}` should be verbatim"
        );
    }
}

/// Fenced blocks are source listings; softening them would corrupt copied code.
#[test]
fn fenced_code_is_left_alone() {
    let Some(text) = render_text("wrapping.md", &Cli::default()) else {
        return;
    };

    assert!(
        text.contains("very_long_identifier_that_must_stay_verbatim"),
        "fenced block contents should be verbatim"
    );
}

/// The template defines helper functions in code mode. A stray newline can drop
/// the parser back into markup and typeset the helper source into the document,
/// which is invisible to a "did it render" smoke test.
#[test]
fn template_source_does_not_leak_into_output() {
    let Some(text) = render_text("wrapping.md", &Cli::default()) else {
        return;
    };

    for marker in [".replace(", "regex(", "captures.at(", "sys.inputs"] {
        assert!(
            !text.contains(marker),
            "template source leaked into the document: found `{marker}`"
        );
    }
}

/// `--number-sections` must reach the document. A `set` rule nested inside an
/// `if` block is scoped to that block and silently does nothing.
#[test]
fn number_sections_numbers_headings() {
    let cli = Cli {
        number_sections: true,
        ..Cli::default()
    };
    let Some(numbered) = render_text("headings.md", &cli) else {
        return;
    };
    let Some(plain) = render_text("headings.md", &Cli::default()) else {
        return;
    };

    assert!(
        numbered.contains("1.1"),
        "numbered headings should carry a `1.1`-style numbering"
    );
    assert!(
        !plain.contains("1.1"),
        "default output should not number headings"
    );
}
