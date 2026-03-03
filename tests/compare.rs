//! Pandoc reference comparison tests.
//!
//! These tests are `#[ignore = "requires pdftotext on PATH"]` by default because they require `pdftotext`
//! on PATH. Run with: `cargo test -- --ignored`

use std::path::Path;
use std::process::Command;

use mdpdf::cli::Cli;
use mdpdf::render::{default_output_path, render_one};

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn reference_text(name: &str) -> std::path::PathBuf {
    let stem = Path::new(name).file_stem().unwrap().to_str().unwrap();
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/references/text")
        .join(format!("{stem}.txt"))
}

fn has_pdftotext() -> bool {
    Command::new("pdftotext").arg("-v").output().is_ok()
}

fn extract_text(pdf_path: &Path) -> String {
    let output = Command::new("pdftotext")
        .arg(pdf_path)
        .arg("-")
        .output()
        .expect("pdftotext failed");
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn normalize(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        // Strip page numbers (bare digits on a line)
        .filter(|l| l.parse::<u32>().is_err())
        .collect::<Vec<_>>()
        .join("\n")
        .to_lowercase()
}

fn compare_fixture(name: &str) {
    if !has_pdftotext() {
        eprintln!("SKIP: pdftotext not found on PATH");
        return;
    }

    let ref_path = reference_text(name);
    if !ref_path.exists() {
        eprintln!("SKIP: reference text not found: {}", ref_path.display());
        return;
    }

    let input = fixture(name);
    let dir = tempfile::tempdir().expect("tempdir");
    let output = dir.path().join(default_output_path(Path::new(name)));
    let cli = Cli::default();
    let result = render_one(&input, &output, &cli);
    assert!(result.success, "{name}: {:?}", result.error);

    let typst_text = normalize(&extract_text(&output));
    let pandoc_text = normalize(&std::fs::read_to_string(&ref_path).expect("read reference"));

    // Report metrics
    #[allow(clippy::cast_precision_loss)]
    let len_ratio = typst_text.len() as f64 / pandoc_text.len().max(1) as f64;
    eprintln!("  {name}: text length ratio (typst/pandoc) = {len_ratio:.2}");

    // Check that the typst output contains most of the pandoc reference content
    // (fuzzy: we check that key content words appear)
    let pandoc_words: Vec<&str> = pandoc_text.split_whitespace().collect();
    let typst_full = typst_text.clone();
    let mut found = 0;
    for word in &pandoc_words {
        if typst_full.contains(word) {
            found += 1;
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let word_overlap = f64::from(found) / pandoc_words.len().max(1) as f64;
    eprintln!("  {name}: word overlap = {word_overlap:.1}%");

    // We expect at least 50% word overlap (generous threshold —
    // different renderers produce different formatting)
    assert!(
        word_overlap > 0.5,
        "{name}: word overlap too low: {word_overlap:.1}% ({found}/{} words)",
        pandoc_words.len()
    );
}

#[test]
#[ignore = "requires pdftotext on PATH"]
fn compare_basic() {
    compare_fixture("basic.md");
}

#[test]
#[ignore = "requires pdftotext on PATH"]
fn compare_math() {
    compare_fixture("math.md");
}

#[test]
#[ignore = "requires pdftotext on PATH"]
fn compare_unicode() {
    compare_fixture("unicode.md");
}

#[test]
#[ignore = "requires pdftotext on PATH"]
fn compare_tables() {
    compare_fixture("tables.md");
}

#[test]
#[ignore = "requires pdftotext on PATH"]
fn compare_code() {
    compare_fixture("code.md");
}

#[test]
#[ignore = "requires pdftotext on PATH"]
fn compare_headings() {
    compare_fixture("headings.md");
}

#[test]
#[ignore = "requires pdftotext on PATH"]
fn compare_long() {
    compare_fixture("long.md");
}
