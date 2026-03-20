use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::Instant;

use anyhow::{Context, Result};
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Dict, IntoValue};
use typst::layout::PagedDocument;
use typst::syntax::{FileId, Source};
use typst_as_lib::TypstEngine;
use typst_as_lib::file_resolver::FileResolver;
use typst_as_lib::typst_kit_options::TypstKitFontOptions;
use typst_embedded_package::{self as tep, Package, include_package};
use typst_pdf::PdfOptions;

use crate::cli::Cli;
use crate::report::RenderResult;

const TEMPLATE: &str = include_str!("template.typ");

static PACKAGES: LazyLock<[Package; 2]> = LazyLock::new(|| {
    include_package!(
        "typst-packages"
        [
            "preview" "cmarker" (0, 1, 8),
            "preview" "mitex" (0, 2, 6),
        ]
    )
});

/// Resolves files from embedded typst packages.
struct EmbeddedPackageResolver {
    sources: HashMap<FileId, Source>,
    binaries: HashMap<FileId, Bytes>,
}

impl EmbeddedPackageResolver {
    fn new(packages: &[Package]) -> Self {
        let mut sources = HashMap::new();
        let mut binaries = HashMap::new();

        for pkg in packages {
            let files = pkg
                .read_archive()
                .expect("embedded package archive should be readable");
            for file in files {
                match file {
                    tep::File::Source(source) => {
                        sources.insert(source.id(), source);
                    }
                    tep::File::File(id, bytes) => {
                        binaries.insert(id, bytes);
                    }
                }
            }
        }

        Self { sources, binaries }
    }
}

impl FileResolver for EmbeddedPackageResolver {
    fn resolve_binary(&self, id: FileId) -> FileResult<Cow<'_, Bytes>> {
        self.binaries
            .get(&id)
            .map(Cow::Borrowed)
            .ok_or_else(|| FileError::NotFound(id.vpath().as_rootless_path().into()))
    }

    fn resolve_source(&self, id: FileId) -> FileResult<Cow<'_, Source>> {
        self.sources
            .get(&id)
            .map(Cow::Borrowed)
            .ok_or_else(|| FileError::NotFound(id.vpath().as_rootless_path().into()))
    }
}

fn build_inputs(content: &str, cli: &Cli) -> Dict {
    let mut dict = Dict::new();
    dict.insert("content".into(), content.into_value());
    dict.insert("margin".into(), cli.margin.as_str().into_value());
    dict.insert("font-size".into(), cli.font_size.as_str().into_value());
    dict.insert("toc".into(), cli.toc.to_string().as_str().into_value());
    dict.insert(
        "number-sections".into(),
        cli.number_sections.to_string().as_str().into_value(),
    );
    dict
}

/// Strip YAML front-matter (a `---`-delimited block at the start of the file).
fn strip_front_matter(content: &str) -> &str {
    let trimmed = content.trim_start();
    let Some(after_open) = trimmed.strip_prefix("---") else {
        return content;
    };
    // Find the closing `---` (must be on its own line after the opening)
    if let Some((_, rest)) = after_open.split_once("\n---") {
        rest.strip_prefix('\n').unwrap_or(rest)
    } else {
        content
    }
}

/// Replace LaTeX commands that have broken mitex symbol mappings with
/// their correct Unicode equivalents.
///
/// mitex 0.2.6 maps `\dashrightarrow` to the invalid typst symbol
/// `arrow.r.dash` (should be `arrow.r.dashed`), and likewise
/// `\dashleftarrow` to `arrow.l.dash`.  Since the mapping is baked
/// into the mitex WASM binary, we fix it here by replacing the LaTeX
/// commands with the corresponding Unicode characters before the
/// content reaches the cmarker/mitex pipeline.
fn fix_mitex_symbols(content: &str) -> String {
    content
        .replace("\\dashrightarrow", "\u{21E2}")
        .replace("\\dashleftarrow", "\u{21E0}")
}

struct CompileOutput {
    pdf: Vec<u8>,
    warnings: Vec<String>,
}

fn compile_to_pdf(content: &str, cli: &Cli) -> Result<CompileOutput> {
    let content = strip_front_matter(content);
    let content = &fix_mitex_symbols(content);
    let inputs = build_inputs(content, cli);

    // Read optional preamble
    let preamble = if let Some(ref path) = cli.include_preamble {
        let extra = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read preamble file: {}", path.display()))?;
        format!("{extra}\n")
    } else {
        String::new()
    };

    let full_template = format!("{preamble}{TEMPLATE}");

    // Build package resolver
    let pkg_resolver = EmbeddedPackageResolver::new(&*PACKAGES);

    // Build engine with embedded fonts and packages
    let font_opts = TypstKitFontOptions::new().include_system_fonts(false);
    let engine = TypstEngine::builder()
        .main_file(("main.typ", full_template.as_str()))
        .search_fonts_with(font_opts)
        .add_file_resolver(pkg_resolver)
        .build();

    // Compile with inputs
    let compiled = engine.compile_with_input(inputs);

    let warnings: Vec<String> = if cli.verbose {
        compiled
            .warnings
            .iter()
            .map(|w| w.message.to_string())
            .collect()
    } else {
        Vec::new()
    };

    let document: PagedDocument = compiled
        .output
        .map_err(|e| anyhow::anyhow!("typst compilation failed:\n  {e}"))?;

    // Export to PDF
    let pdf_bytes = typst_pdf::pdf(&document, &PdfOptions::default()).map_err(|diagnostics| {
        let mut msg = String::from("PDF export failed:");
        for diag in &diagnostics {
            let _ = write!(msg, "\n  error: {}", diag.message);
        }
        anyhow::anyhow!("{msg}")
    })?;

    Ok(CompileOutput {
        pdf: pdf_bytes,
        warnings,
    })
}

/// Return the Typst source that would be compiled, prefixed with a
/// `sys.inputs` header. Used by `--dry-run` to inspect the template
/// without invoking the Typst compiler.
#[must_use]
pub fn format_dry_run(content: &str, cli: &Cli) -> String {
    let content = strip_front_matter(content);
    let mut parts = vec![String::from("// sys.inputs:")];

    let inputs_display: Vec<(&str, String)> = vec![
        ("content", format!("<{} bytes>", content.len())),
        ("font-size", cli.font_size.clone()),
        ("margin", cli.margin.clone()),
        ("number-sections", cli.number_sections.to_string()),
        ("toc", cli.toc.to_string()),
    ];

    for (k, v) in &inputs_display {
        parts.push(format!("//   {k}: {v}"));
    }
    parts.push(String::new());

    if let Some(ref path) = cli.include_preamble {
        parts.push(format!("// preamble: {}", path.display()));
        if let Ok(extra) = std::fs::read_to_string(path) {
            parts.push(extra);
            parts.push(String::new());
        }
    }

    parts.push(TEMPLATE.to_string());
    parts.join("\n")
}

/// Replace the input file's extension with `.pdf`.
#[must_use]
pub fn default_output_path(input: &Path) -> PathBuf {
    input.with_extension("pdf")
}

/// Read a single markdown file, compile it to PDF via Typst, and write
/// the result. Returns a [`RenderResult`] (never panics).
#[must_use]
pub fn render_one(input: &Path, output: &Path, cli: &Cli) -> RenderResult {
    let start = Instant::now();
    let inp = input.display().to_string();
    let out = output.display().to_string();
    let r = RenderResult::builder(&inp, &out, &start);

    let content = match std::fs::read_to_string(input) {
        Ok(c) => c,
        Err(e) => return r.fail(&format!("failed to read {}: {e}", input.display())),
    };

    match compile_to_pdf(&content, cli) {
        Ok(compiled) => {
            let r = r.warnings(compiled.warnings);
            if let Err(e) = std::fs::write(output, &compiled.pdf) {
                return r.fail(&format!("failed to write {}: {e}", output.display()));
            }
            r.ok()
        }
        Err(e) => r.fail(&e),
    }
}

/// Read markdown from stdin, compile to PDF, and write the result.
/// Returns a [`RenderResult`] (never panics), matching [`render_one`].
#[must_use]
pub fn render_stdin(output: &Path, cli: &Cli) -> RenderResult {
    let start = Instant::now();
    let out = output.display().to_string();
    let r = RenderResult::builder("<stdin>", &out, &start);

    let mut content = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut content) {
        return r.fail(&format!("failed to read stdin: {e}"));
    }

    if content.is_empty() {
        return r.fail(&"no input on stdin");
    }

    match compile_to_pdf(&content, cli) {
        Ok(compiled) => {
            let r = r.warnings(compiled.warnings);
            if let Err(e) = std::fs::write(output, &compiled.pdf) {
                return r.fail(&format!("failed to write {}: {e}", output.display()));
            }
            r.ok()
        }
        Err(e) => r.fail(&e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn default_output_path_md_to_pdf() {
        assert_eq!(
            default_output_path(Path::new("doc.md")),
            PathBuf::from("doc.pdf")
        );
    }

    #[test]
    fn default_output_path_nested() {
        assert_eq!(
            default_output_path(Path::new("/tmp/foo/bar.md")),
            PathBuf::from("/tmp/foo/bar.pdf")
        );
    }

    #[test]
    fn default_output_path_no_extension() {
        assert_eq!(
            default_output_path(Path::new("README")),
            PathBuf::from("README.pdf")
        );
    }

    #[test]
    fn dry_run_contains_sys_inputs() {
        let cli = Cli::default();
        let output = format_dry_run("hello world", &cli);

        assert!(output.contains("// sys.inputs:"));
        assert!(output.contains("content"));
        assert!(output.contains("font-size"));
        assert!(output.contains("margin"));
        assert!(output.contains("toc"));
        assert!(output.contains("number-sections"));
    }

    #[test]
    fn dry_run_contains_template() {
        let cli = Cli::default();
        let output = format_dry_run("test", &cli);

        assert!(output.contains("cmarker"));
        assert!(output.contains("mitex"));
    }

    #[test]
    fn dry_run_shows_content_size() {
        let cli = Cli::default();
        let content = "x".repeat(42);
        let output = format_dry_run(&content, &cli);

        assert!(output.contains("<42 bytes>"));
    }

    #[test]
    fn dry_run_custom_margin() {
        let cli = Cli {
            margin: "0.5in".to_string(),
            ..Cli::default()
        };
        let output = format_dry_run("test", &cli);

        assert!(output.contains("0.5in"));
    }

    #[test]
    fn strip_front_matter_removes_yaml() {
        let input = "---\ntitle: Hello\ndate: 2026\n---\n# Heading\n";
        assert_eq!(strip_front_matter(input), "# Heading\n");
    }

    #[test]
    fn strip_front_matter_no_front_matter() {
        let input = "# Just a heading\nSome text.\n";
        assert_eq!(strip_front_matter(input), input);
    }

    #[test]
    fn strip_front_matter_unclosed() {
        let input = "---\ntitle: Hello\n# No closing delimiter\n";
        assert_eq!(strip_front_matter(input), input);
    }

    #[test]
    fn strip_front_matter_hr_not_confused() {
        // A `---` that isn't at the start shouldn't be stripped
        let input = "# Title\n\n---\n\nSome text.\n";
        assert_eq!(strip_front_matter(input), input);
    }

    #[test]
    fn dry_run_strips_front_matter() {
        let cli = Cli::default();
        let content = "---\ntitle: Test\n---\nHello world";
        let output = format_dry_run(content, &cli);
        // Size should reflect stripped content, not original
        assert!(output.contains("<11 bytes>"));
    }

    #[test]
    fn fix_mitex_symbols_dashrightarrow() {
        let input = r"$$a \dashrightarrow b$$";
        let result = fix_mitex_symbols(input);
        assert!(result.contains('\u{21E2}'));
        assert!(!result.contains("dashrightarrow"));
    }

    #[test]
    fn fix_mitex_symbols_dashleftarrow() {
        let input = r"$$a \dashleftarrow b$$";
        let result = fix_mitex_symbols(input);
        assert!(result.contains('\u{21E0}'));
        assert!(!result.contains("dashleftarrow"));
    }

    #[test]
    fn fix_mitex_symbols_no_false_positives() {
        let input = r"$$a \rightarrow b$$";
        let result = fix_mitex_symbols(input);
        assert_eq!(result, input);
    }
}
