use std::time::Instant;

use serde::Serialize;

/// Outcome of rendering a single file (or stdin). Serialises to JSONL
/// for `--json` mode; also drives human-readable stderr output.
#[derive(Debug, Serialize)]
pub struct RenderResult {
    pub input: String,
    pub output: String,
    pub success: bool,
    pub time_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

impl RenderResult {
    /// Start building a result for the given input/output pair.
    #[must_use]
    pub fn builder<'a>(input: &str, output: &str, start: &'a Instant) -> RenderResultBuilder<'a> {
        RenderResultBuilder {
            input: input.to_string(),
            output: output.to_string(),
            start,
            warnings: Vec::new(),
        }
    }

    /// Print a one-line summary to stderr (` ok:` or ` FAIL:`),
    /// followed by a warnings block if any are present.
    pub fn print_human(&self) {
        if self.success {
            eprintln!("  ok: {} ({}ms)", self.output, self.time_ms);
        } else {
            eprintln!("  FAIL: {}", self.input);
            if let Some(err) = &self.error {
                for line in err.lines().take(10) {
                    eprintln!("    {line}");
                }
            }
        }
        if !self.warnings.is_empty() {
            eprintln!("  warnings:");
            for w in &self.warnings {
                eprintln!("    {w}");
            }
        }
    }

    /// Serialise this result as a single JSON line to stdout.
    pub fn print_json(&self) {
        match serde_json::to_string(self) {
            Ok(json) => println!("{json}"),
            Err(e) => eprintln!("error: failed to serialize result: {e}"),
        }
    }
}

pub struct RenderResultBuilder<'a> {
    input: String,
    output: String,
    start: &'a Instant,
    warnings: Vec<String>,
}

impl RenderResultBuilder<'_> {
    #[must_use]
    pub fn warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }

    #[must_use]
    pub fn ok(self) -> RenderResult {
        let time_ms = elapsed_ms(self.start);
        RenderResult {
            input: self.input,
            output: self.output,
            success: true,
            time_ms,
            error: None,
            warnings: self.warnings,
        }
    }

    #[must_use]
    pub fn fail(self, error: &impl ToString) -> RenderResult {
        let time_ms = elapsed_ms(self.start);
        RenderResult {
            input: self.input,
            output: self.output,
            success: false,
            time_ms,
            error: Some(error.to_string()),
            warnings: self.warnings,
        }
    }
}

fn elapsed_ms(start: &Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}
