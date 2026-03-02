use std::path::PathBuf;

use clap::Parser;

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
    mdpdf doc.md --no-toc           Skip table of contents generation
    mdpdf doc.md --margin 0.75in    Custom margins")]
pub struct Cli {
    /// Markdown files to convert. Reads from stdin if none given.
    pub files: Vec<PathBuf>,

    /// Output file path. Only valid with a single input file or stdin.
    /// Default: input stem + .pdf in the same directory.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Generate table of contents.
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub toc: bool,

    /// Disable table of contents.
    #[arg(long)]
    pub no_toc: bool,

    /// Number document sections.
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub number_sections: bool,

    /// Disable section numbering.
    #[arg(long)]
    pub no_number_sections: bool,

    /// Page margin (e.g. 1in, 0.75in, 2cm).
    #[arg(long, default_value = "1in")]
    pub margin: String,

    /// Font size (e.g. 10pt, 11pt, 12pt).
    #[arg(long, default_value = "11pt")]
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

    /// Maximum number of parallel render jobs.
    #[arg(long, short = 'J', default_value = "8")]
    pub jobs: usize,
}

impl Cli {
    pub fn toc_enabled(&self) -> bool {
        if self.no_toc {
            return false;
        }
        self.toc
    }

    pub fn number_sections_enabled(&self) -> bool {
        if self.no_number_sections {
            return false;
        }
        self.number_sections
    }
}
