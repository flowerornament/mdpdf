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
    #[must_use]
    pub fn toc_enabled(&self) -> bool {
        if self.no_toc {
            return false;
        }
        self.toc
    }

    #[must_use]
    pub fn number_sections_enabled(&self) -> bool {
        if self.no_number_sections {
            return false;
        }
        self.number_sections
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn toc_enabled_by_default() {
        let cli = default_cli();
        assert!(cli.toc_enabled());
    }

    #[test]
    fn toc_disabled_by_no_toc() {
        let mut cli = default_cli();
        cli.no_toc = true;
        assert!(!cli.toc_enabled());
    }

    #[test]
    fn toc_disabled_when_toc_false() {
        let mut cli = default_cli();
        cli.toc = false;
        assert!(!cli.toc_enabled());
    }

    #[test]
    fn no_toc_overrides_toc_true() {
        let mut cli = default_cli();
        cli.toc = true;
        cli.no_toc = true;
        assert!(!cli.toc_enabled());
    }

    #[test]
    fn number_sections_enabled_by_default() {
        let cli = default_cli();
        assert!(cli.number_sections_enabled());
    }

    #[test]
    fn number_sections_disabled_by_no_flag() {
        let mut cli = default_cli();
        cli.no_number_sections = true;
        assert!(!cli.number_sections_enabled());
    }

    #[test]
    fn no_number_sections_overrides_true() {
        let mut cli = default_cli();
        cli.number_sections = true;
        cli.no_number_sections = true;
        assert!(!cli.number_sections_enabled());
    }
}
