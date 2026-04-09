use std::path::PathBuf;

use clap::Parser;

fn parse_dimension(s: &str) -> Result<String, String> {
    let s = s.trim();
    for unit in ["in", "cm", "mm", "pt", "em"] {
        if let Some(num) = s.strip_suffix(unit)
            && num.trim().parse::<f64>().is_ok()
        {
            return Ok(s.to_string());
        }
    }
    Err(format!(
        "invalid dimension '{s}': expected <number><unit> (e.g. 1in, 0.75in, 2cm, 11pt)"
    ))
}

/// Markdown-to-PDF transducer with built-in unicode math support.
///
/// Converts markdown files to beautifully typeset PDFs using typst.
/// Handles unicode math, Greek letters, and special characters natively
/// with zero external dependencies.
#[derive(Parser, Debug)]
#[command(version, about)]
#[allow(clippy::struct_excessive_bools)]
#[command(after_help = "\
EXAMPLES:
    mdpdf doc.md                    Convert a single file (outputs doc.pdf)
    mdpdf doc.md -o out.pdf         Convert with explicit output path
    mdpdf a.md b.md c.md            Convert multiple files in parallel
    mdpdf *.md --json               Batch convert with JSONL output
    cat doc.md | mdpdf -o doc.pdf   Convert from stdin
    mdpdf doc.md --dry-run          Print generated typst source
    mdpdf doc.md --toc              Include table of contents
    mdpdf doc.md --margin 0.75in    Custom margins")]
pub struct Cli {
    /// Markdown files to convert. Reads from stdin if none given.
    pub files: Vec<PathBuf>,

    /// Output file path. Only valid with a single input file or stdin.
    /// Default: input stem + .pdf in the same directory.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Generate table of contents.
    #[arg(long)]
    pub toc: bool,

    /// Number document sections.
    #[arg(long)]
    pub number_sections: bool,

    /// Page margin (e.g. 1in, 0.75in, 2cm).
    #[arg(long, default_value = "1in", value_parser = parse_dimension)]
    pub margin: String,

    /// Font size (e.g. 10pt, 11pt, 12pt).
    #[arg(long, default_value = "11pt", value_parser = parse_dimension)]
    pub font_size: String,

    /// Additional typst code to include before the template.
    #[arg(long)]
    pub include_preamble: Option<PathBuf>,

    /// Output JSONL structured results (one JSON object per file).
    #[arg(long, short = 'j')]
    pub json: bool,

    /// Print generated typst source without rendering.
    #[arg(long)]
    pub dry_run: bool,

    /// Show typst warnings during compilation.
    #[arg(long, short = 'v')]
    pub verbose: bool,

    /// Maximum number of parallel render jobs.
    #[arg(long, short = 'J', default_value = "8")]
    pub jobs: usize,
}

impl Default for Cli {
    fn default() -> Self {
        Self::try_parse_from(["mdpdf"]).expect("default CLI args should parse")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let cli = Cli::try_parse_from(["mdpdf", "file.md"]).unwrap();
        assert!(!cli.toc);
        assert!(!cli.number_sections);
        assert_eq!(cli.margin, "1in");
        assert_eq!(cli.font_size, "11pt");
        assert_eq!(cli.jobs, 8);
    }

    #[test]
    fn toc_flag() {
        let cli = Cli::try_parse_from(["mdpdf", "--toc", "file.md"]).unwrap();
        assert!(cli.toc);
    }

    #[test]
    fn number_sections_flag() {
        let cli = Cli::try_parse_from(["mdpdf", "--number-sections", "file.md"]).unwrap();
        assert!(cli.number_sections);
    }

    #[test]
    fn custom_margin() {
        let cli = Cli::try_parse_from(["mdpdf", "--margin", "0.5in", "file.md"]).unwrap();
        assert_eq!(cli.margin, "0.5in");
    }

    #[test]
    fn output_flag() {
        let cli = Cli::try_parse_from(["mdpdf", "-o", "out.pdf", "file.md"]).unwrap();
        assert_eq!(cli.output, Some(PathBuf::from("out.pdf")));
    }

    #[test]
    fn multiple_files() {
        let cli = Cli::try_parse_from(["mdpdf", "a.md", "b.md", "c.md"]).unwrap();
        assert_eq!(cli.files.len(), 3);
    }
}
