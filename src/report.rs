use serde::Serialize;

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

    pub fn print_json(&self) {
        if let Ok(json) = serde_json::to_string(self) {
            println!("{json}");
        }
    }
}
