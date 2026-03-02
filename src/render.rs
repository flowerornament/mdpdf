use std::borrow::Cow;
use std::collections::HashMap;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::Instant;

use anyhow::{Context, Result, bail};
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
            if let Ok(files) = pkg.read_archive() {
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

fn elapsed_ms(start: &Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn build_inputs(content: &str, cli: &Cli) -> Dict {
    let mut dict = Dict::new();
    dict.insert("content".into(), content.into_value());
    dict.insert("margin".into(), cli.margin.as_str().into_value());
    dict.insert("font-size".into(), cli.font_size.as_str().into_value());
    dict.insert(
        "toc".into(),
        cli.toc_enabled().to_string().as_str().into_value(),
    );
    dict.insert(
        "number-sections".into(),
        cli.number_sections_enabled()
            .to_string()
            .as_str()
            .into_value(),
    );
    dict
}

fn compile_to_pdf(content: &str, cli: &Cli) -> Result<Vec<u8>> {
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

    // Log warnings
    for warning in &compiled.warnings {
        eprintln!("  typst warning: {warning:?}");
    }

    let document: PagedDocument = compiled
        .output
        .map_err(|e| anyhow::anyhow!("typst compilation failed: {e}"))?;

    // Export to PDF
    let pdf_bytes = typst_pdf::pdf(&document, &PdfOptions::default()).map_err(|diagnostics| {
        let msgs: Vec<String> = diagnostics.iter().map(|d| format!("{d:?}")).collect();
        anyhow::anyhow!("PDF export failed:\n{}", msgs.join("\n"))
    })?;

    Ok(pdf_bytes)
}

/// Return the Typst source that would be compiled, prefixed with a
/// `sys.inputs` header. Used by `--dry-run` to inspect the template
/// without invoking the Typst compiler.
#[must_use]
pub fn format_dry_run(content: &str, cli: &Cli) -> String {
    let mut parts = vec![String::from("// sys.inputs:")];

    let inputs_display: Vec<(&str, String)> = vec![
        ("content", format!("<{} bytes>", content.len())),
        ("font-size", cli.font_size.clone()),
        ("margin", cli.margin.clone()),
        ("number-sections", cli.number_sections_enabled().to_string()),
        ("toc", cli.toc_enabled().to_string()),
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

    let content = match std::fs::read_to_string(input) {
        Ok(c) => c,
        Err(e) => {
            return RenderResult {
                input: input.display().to_string(),
                output: output.display().to_string(),
                success: false,
                time_ms: elapsed_ms(&start),
                error: Some(format!("failed to read {}: {e}", input.display())),
            };
        }
    };

    match compile_to_pdf(&content, cli) {
        Ok(pdf_bytes) => {
            if let Err(e) = std::fs::write(output, &pdf_bytes) {
                return RenderResult {
                    input: input.display().to_string(),
                    output: output.display().to_string(),
                    success: false,
                    time_ms: elapsed_ms(&start),
                    error: Some(format!("failed to write {}: {e}", output.display())),
                };
            }
            RenderResult {
                input: input.display().to_string(),
                output: output.display().to_string(),
                success: true,
                time_ms: elapsed_ms(&start),
                error: None,
            }
        }
        Err(e) => RenderResult {
            input: input.display().to_string(),
            output: output.display().to_string(),
            success: false,
            time_ms: elapsed_ms(&start),
            error: Some(e.to_string()),
        },
    }
}

/// # Errors
/// Returns an error if stdin cannot be read or is empty.
pub fn render_stdin(output: &Path, cli: &Cli) -> Result<RenderResult> {
    let start = Instant::now();

    let mut content = String::new();
    io::stdin()
        .read_to_string(&mut content)
        .context("failed to read stdin")?;

    if content.is_empty() {
        bail!("no input on stdin");
    }

    match compile_to_pdf(&content, cli) {
        Ok(pdf_bytes) => {
            std::fs::write(output, &pdf_bytes)
                .with_context(|| format!("failed to write {}", output.display()))?;
            Ok(RenderResult {
                input: "<stdin>".to_string(),
                output: output.display().to_string(),
                success: true,
                time_ms: elapsed_ms(&start),
                error: None,
            })
        }
        Err(e) => Ok(RenderResult {
            input: "<stdin>".to_string(),
            output: output.display().to_string(),
            success: false,
            time_ms: elapsed_ms(&start),
            error: Some(e.to_string()),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

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
        let cli = default_cli();
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
        let cli = default_cli();
        let output = format_dry_run("test", &cli);

        assert!(output.contains("cmarker"));
        assert!(output.contains("mitex"));
    }

    #[test]
    fn dry_run_shows_content_size() {
        let cli = default_cli();
        let content = "x".repeat(42);
        let output = format_dry_run(&content, &cli);

        assert!(output.contains("<42 bytes>"));
    }

    #[test]
    fn dry_run_custom_margin() {
        let mut cli = default_cli();
        cli.margin = "0.5in".to_string();
        let output = format_dry_run("test", &cli);

        assert!(output.contains("0.5in"));
    }
}
