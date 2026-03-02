use std::path::Path;

use mdpdf::cli::Cli;
use mdpdf::render::{default_output_path, format_dry_run, render_one};

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn default_cli() -> Cli {
    Cli {
        files: vec![],
        output: None,
        toc: true,
        no_toc: false,
        number_sections: true,
        no_number_sections: false,
        margin: "1in".to_string(),
        font_size: "11pt".to_string(),
        include_preamble: None,
        json: false,
        dry_run: false,
        jobs: 8,
    }
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
    let result = render_fixture("basic.md", &default_cli());
    assert!(result.success, "basic.md: {:?}", result.error);
}

#[test]
fn render_math() {
    let result = render_fixture("math.md", &default_cli());
    assert!(result.success, "math.md: {:?}", result.error);
}

#[test]
fn render_unicode() {
    let result = render_fixture("unicode.md", &default_cli());
    assert!(result.success, "unicode.md: {:?}", result.error);
}

#[test]
fn render_tables() {
    let result = render_fixture("tables.md", &default_cli());
    assert!(result.success, "tables.md: {:?}", result.error);
}

#[test]
fn render_code() {
    let result = render_fixture("code.md", &default_cli());
    assert!(result.success, "code.md: {:?}", result.error);
}

#[test]
fn render_headings() {
    let result = render_fixture("headings.md", &default_cli());
    assert!(result.success, "headings.md: {:?}", result.error);
}

#[test]
fn render_long() {
    let result = render_fixture("long.md", &default_cli());
    assert!(result.success, "long.md: {:?}", result.error);
}

// --- Feature tests ---

#[test]
fn no_toc_produces_smaller_output() {
    let input = fixture("long.md");
    let dir = tempfile::tempdir().expect("tempdir");

    // With TOC
    let out_toc = dir.path().join("with_toc.pdf");
    let cli_toc = default_cli();
    let r1 = render_one(&input, &out_toc, &cli_toc);
    assert!(r1.success, "with toc: {:?}", r1.error);

    // Without TOC
    let out_no_toc = dir.path().join("no_toc.pdf");
    let mut cli_no_toc = default_cli();
    cli_no_toc.no_toc = true;
    let r2 = render_one(&input, &out_no_toc, &cli_no_toc);
    assert!(r2.success, "no toc: {:?}", r2.error);

    let size_toc = std::fs::metadata(&out_toc).unwrap().len();
    let size_no_toc = std::fs::metadata(&out_no_toc).unwrap().len();
    assert!(
        size_no_toc < size_toc,
        "no-toc PDF ({size_no_toc}) should be smaller than toc PDF ({size_toc})"
    );
}

#[test]
fn custom_margin_renders() {
    let mut cli = default_cli();
    cli.margin = "0.5in".to_string();
    let result = render_fixture("basic.md", &cli);
    assert!(result.success, "custom margin: {:?}", result.error);
}

#[test]
fn custom_font_size_renders() {
    let mut cli = default_cli();
    cli.font_size = "14pt".to_string();
    let result = render_fixture("basic.md", &cli);
    assert!(result.success, "custom font size: {:?}", result.error);
}

// --- Error cases ---

#[test]
fn missing_file_returns_failure() {
    let cli = default_cli();
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
    let cli = default_cli();
    let content = std::fs::read_to_string(fixture("basic.md")).expect("read fixture");
    let output = format_dry_run(&content, &cli);
    assert!(output.contains("cmarker"));
    assert!(output.contains("sys.inputs"));
}

// --- JSON round-trip ---

#[test]
fn render_result_json_roundtrip() {
    let result = render_fixture("basic.md", &default_cli());
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
