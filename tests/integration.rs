use std::path::Path;

use mdpdf::cli::Cli;
use mdpdf::render::{default_output_path, format_dry_run, render_one};

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn render_fixture(name: &str, cli: &Cli) -> mdpdf::report::RenderResult {
    let input = fixture(name);
    let dir = tempfile::tempdir().expect("tempdir");
    let output = dir.path().join(default_output_path(Path::new(name)));
    let result = render_one(&input, &output, cli);
    // Keep tempdir alive long enough to check the file
    if result.success {
        assert!(output.exists(), "output file should exist: {name}");
        let meta = std::fs::metadata(&output).expect("metadata");
        assert!(meta.len() > 0, "output file should be non-empty: {name}");
    }
    result
}

// --- Each fixture renders successfully ---

#[test]
fn render_basic() {
    let result = render_fixture("basic.md", &Cli::default());
    assert!(result.success, "basic.md: {:?}", result.error);
}

#[test]
fn render_math() {
    let result = render_fixture("math.md", &Cli::default());
    assert!(result.success, "math.md: {:?}", result.error);
}

#[test]
fn render_unicode() {
    let result = render_fixture("unicode.md", &Cli::default());
    assert!(result.success, "unicode.md: {:?}", result.error);
}

#[test]
fn render_tables() {
    let result = render_fixture("tables.md", &Cli::default());
    assert!(result.success, "tables.md: {:?}", result.error);
}

#[test]
fn render_code() {
    let result = render_fixture("code.md", &Cli::default());
    assert!(result.success, "code.md: {:?}", result.error);
}

#[test]
fn render_headings() {
    let result = render_fixture("headings.md", &Cli::default());
    assert!(result.success, "headings.md: {:?}", result.error);
}

#[test]
fn render_long() {
    let result = render_fixture("long.md", &Cli::default());
    assert!(result.success, "long.md: {:?}", result.error);
}

// --- Feature tests ---

#[test]
fn toc_produces_larger_output() {
    let input = fixture("long.md");
    let dir = tempfile::tempdir().expect("tempdir");

    // Without TOC (default)
    let out_no_toc = dir.path().join("no_toc.pdf");
    let cli_no_toc = Cli::default();
    let r1 = render_one(&input, &out_no_toc, &cli_no_toc);
    assert!(r1.success, "no toc: {:?}", r1.error);

    // With TOC
    let out_toc = dir.path().join("with_toc.pdf");
    let cli_toc = Cli {
        toc: true,
        ..Cli::default()
    };
    let r2 = render_one(&input, &out_toc, &cli_toc);
    assert!(r2.success, "with toc: {:?}", r2.error);

    let size_no_toc = std::fs::metadata(&out_no_toc).unwrap().len();
    let size_toc = std::fs::metadata(&out_toc).unwrap().len();
    assert!(
        size_toc > size_no_toc,
        "toc PDF ({size_toc}) should be larger than no-toc PDF ({size_no_toc})"
    );
}

#[test]
fn custom_margin_renders() {
    let cli = Cli {
        margin: "0.5in".to_string(),
        ..Cli::default()
    };
    let result = render_fixture("basic.md", &cli);
    assert!(result.success, "custom margin: {:?}", result.error);
}

#[test]
fn custom_font_size_renders() {
    let cli = Cli {
        font_size: "14pt".to_string(),
        ..Cli::default()
    };
    let result = render_fixture("basic.md", &cli);
    assert!(result.success, "custom font size: {:?}", result.error);
}

// --- Error cases ---

#[test]
fn missing_file_returns_failure() {
    let cli = Cli::default();
    let input = Path::new("/nonexistent/file.md");
    let dir = tempfile::tempdir().expect("tempdir");
    let output = dir.path().join("out.pdf");
    let result = render_one(input, &output, &cli);
    assert!(!result.success);
    assert!(result.error.is_some());
}

// --- Dry run ---

#[test]
fn dry_run_output_contains_template() {
    let cli = Cli::default();
    let content = std::fs::read_to_string(fixture("basic.md")).expect("read fixture");
    let output = format_dry_run(&content, &cli);
    assert!(output.contains("cmarker"));
    assert!(output.contains("sys.inputs"));
}

// --- JSON round-trip ---

#[test]
fn render_result_json_roundtrip() {
    let result = render_fixture("basic.md", &Cli::default());
    let json = serde_json::to_string(&result).expect("serialize");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("parse");
    assert_eq!(parsed["success"], true);
    assert!(parsed["time_ms"].as_u64().unwrap() > 0);
    assert!(
        std::path::Path::new(parsed["output"].as_str().unwrap())
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
    );
}
