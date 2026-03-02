use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use tempfile::NamedTempFile;

use crate::cli::Cli;
use crate::report::RenderResult;

const PREAMBLE: &str = include_str!("preamble.tex");

fn elapsed_ms(start: &Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

pub fn check_dependencies() -> Result<(), Vec<String>> {
    let mut missing = Vec::new();
    if which::which("pandoc").is_err() {
        missing.push("pandoc".to_string());
    }
    if which::which("tectonic").is_err() {
        missing.push("tectonic".to_string());
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}

fn write_preamble(extra_header: Option<&Path>) -> Result<NamedTempFile> {
    let mut file = NamedTempFile::new().context("failed to create temp preamble file")?;
    file.write_all(PREAMBLE.as_bytes())
        .context("failed to write preamble")?;

    if let Some(header_path) = extra_header {
        let extra = std::fs::read_to_string(header_path)
            .with_context(|| format!("failed to read header file: {}", header_path.display()))?;
        file.write_all(b"\n").context("failed to write newline")?;
        file.write_all(extra.as_bytes())
            .context("failed to write extra header")?;
    }

    file.flush().context("failed to flush preamble")?;
    Ok(file)
}

fn build_pandoc_command(input: &Path, output: &Path, preamble_path: &Path, cli: &Cli) -> Command {
    let mut cmd = Command::new("pandoc");
    cmd.arg(input);
    cmd.arg("-o").arg(output);
    cmd.arg("--pdf-engine=tectonic");
    cmd.arg(format!("-V geometry:margin={}", cli.margin));
    cmd.arg(format!("-V fontsize={}", cli.font_size));
    cmd.arg(format!("-V documentclass={}", cli.document_class));

    if cli.toc_enabled() {
        cmd.arg("--toc");
    }
    if cli.number_sections_enabled() {
        cmd.arg("--number-sections");
    }

    cmd.arg("--include-in-header").arg(preamble_path);
    cmd
}

fn build_stdin_command(output: &Path, preamble_path: &Path, cli: &Cli) -> Command {
    let mut cmd = Command::new("pandoc");
    cmd.arg("-f").arg("markdown");
    cmd.arg("-o").arg(output);
    cmd.arg("--pdf-engine=tectonic");
    cmd.arg(format!("-V geometry:margin={}", cli.margin));
    cmd.arg(format!("-V fontsize={}", cli.font_size));
    cmd.arg(format!("-V documentclass={}", cli.document_class));

    if cli.toc_enabled() {
        cmd.arg("--toc");
    }
    if cli.number_sections_enabled() {
        cmd.arg("--number-sections");
    }

    cmd.arg("--include-in-header").arg(preamble_path);
    cmd.stdin(std::process::Stdio::piped());
    cmd
}

pub fn format_dry_run_command(input: &str, output: &Path, cli: &Cli) -> String {
    let mut parts = vec![
        "pandoc".to_string(),
        input.to_string(),
        "-o".to_string(),
        output.display().to_string(),
        "--pdf-engine=tectonic".to_string(),
        format!("-V geometry:margin={}", cli.margin),
        format!("-V fontsize={}", cli.font_size),
        format!("-V documentclass={}", cli.document_class),
    ];

    if cli.toc_enabled() {
        parts.push("--toc".to_string());
    }
    if cli.number_sections_enabled() {
        parts.push("--number-sections".to_string());
    }

    parts.push("--include-in-header=<preamble>".to_string());
    parts.join(" \\\n  ")
}

pub fn default_output_path(input: &Path) -> PathBuf {
    input.with_extension("pdf")
}

pub fn render_one(input: &Path, output: &Path, cli: &Cli) -> RenderResult {
    let start = Instant::now();

    let preamble = match write_preamble(cli.include_header.as_deref()) {
        Ok(p) => p,
        Err(e) => {
            return RenderResult {
                input: input.display().to_string(),
                output: output.display().to_string(),
                success: false,
                time_ms: elapsed_ms(&start),
                error: Some(e.to_string()),
            };
        }
    };

    let result = build_pandoc_command(input, output, preamble.path(), cli).output();

    let elapsed = elapsed_ms(&start);

    match result {
        Ok(out) if out.status.success() => RenderResult {
            input: input.display().to_string(),
            output: output.display().to_string(),
            success: true,
            time_ms: elapsed,
            error: None,
        },
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            RenderResult {
                input: input.display().to_string(),
                output: output.display().to_string(),
                success: false,
                time_ms: elapsed,
                error: Some(stderr.to_string()),
            }
        }
        Err(e) => RenderResult {
            input: input.display().to_string(),
            output: output.display().to_string(),
            success: false,
            time_ms: elapsed,
            error: Some(e.to_string()),
        },
    }
}

pub fn render_stdin(output: &Path, cli: &Cli) -> Result<RenderResult> {
    let start = Instant::now();

    let mut input_data = Vec::new();
    io::stdin()
        .read_to_end(&mut input_data)
        .context("failed to read stdin")?;

    if input_data.is_empty() {
        bail!("no input on stdin");
    }

    let preamble = write_preamble(cli.include_header.as_deref())?;

    let mut child = build_stdin_command(output, preamble.path(), cli)
        .spawn()
        .context("failed to start pandoc")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&input_data)
            .context("failed to write to pandoc stdin")?;
    }

    let out = child
        .wait_with_output()
        .context("failed to wait for pandoc")?;

    let elapsed = elapsed_ms(&start);

    if out.status.success() {
        Ok(RenderResult {
            input: "<stdin>".to_string(),
            output: output.display().to_string(),
            success: true,
            time_ms: elapsed,
            error: None,
        })
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr);
        Ok(RenderResult {
            input: "<stdin>".to_string(),
            output: output.display().to_string(),
            success: false,
            time_ms: elapsed,
            error: Some(stderr.to_string()),
        })
    }
}
