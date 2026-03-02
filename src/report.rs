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
}

impl RenderResult {
    /// Print a one-line summary to stderr (` ok:` or ` FAIL:`).
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
    }

    /// Serialise this result as a single JSON line to stdout.
    pub fn print_json(&self) {
        if let Ok(json) = serde_json::to_string(self) {
            println!("{json}");
        }
    }
}
