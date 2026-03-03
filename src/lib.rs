//! Markdown-to-PDF renderer powered by Typst.
//!
//! Reads `.md` files, pipes them through a Typst template with cmarker,
//! and writes PDF output. Supports parallel multi-file rendering, stdin,
//! dry-run mode, and JSONL output.
//!
//! - [`cli`] — command-line argument definitions
//! - [`render`] — compilation and PDF export
//! - [`report`] — structured render results

pub mod cli;
pub mod render;
pub mod report;

use std::io::IsTerminal;
use std::process::ExitCode;

use clap::{CommandFactory, Parser};
use rayon::prelude::*;

use cli::Cli;
use render::{default_output_path, format_dry_run, render_one, render_stdin};

/// Parse CLI arguments and render. Entry point for the binary.
#[must_use]
pub fn run() -> ExitCode {
    let cli = Cli::parse();
    run_with(&cli)
}

/// Render with a pre-built [`Cli`]. Testable entry point.
#[must_use]
pub fn run_with(cli: &Cli) -> ExitCode {
    // Stdin mode: no files given
    if cli.files.is_empty() {
        if std::io::stdin().is_terminal() {
            Cli::command().print_help().ok();
            return ExitCode::SUCCESS;
        }
        return run_stdin_mode(cli);
    }

    // Validate: -o only valid with single file
    if cli.output.is_some() && cli.files.len() > 1 {
        eprintln!("error: --output can only be used with a single input file");
        return ExitCode::from(1);
    }

    // Dry-run mode
    if cli.dry_run {
        for file in &cli.files {
            let content = match std::fs::read_to_string(file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("error: failed to read {}: {e}", file.display());
                    return ExitCode::from(1);
                }
            };
            let source = format_dry_run(&content, cli);
            println!("{source}");
            if cli.files.len() > 1 {
                println!();
            }
        }
        return ExitCode::SUCCESS;
    }

    // Single file
    if cli.files.len() == 1 {
        let input = &cli.files[0];
        let output = cli
            .output
            .clone()
            .unwrap_or_else(|| default_output_path(input));
        let result = render_one(input, &output, cli);

        if cli.json {
            result.print_json();
        } else {
            result.print_human();
        }

        return if result.success {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(1)
        };
    }

    // Multi-file parallel: collect results, then print sequentially
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(cli.jobs)
        .build();

    let pool = match pool {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: failed to create thread pool: {e}");
            return ExitCode::from(1);
        }
    };

    let results: Vec<_> = pool.install(|| {
        cli.files
            .par_iter()
            .map(|input| {
                let output = default_output_path(input);
                render_one(input, &output, cli)
            })
            .collect()
    });

    let mut fail_count = 0usize;
    for result in &results {
        if !result.success {
            fail_count += 1;
        }
        if cli.json {
            result.print_json();
        } else {
            result.print_human();
        }
    }

    if !cli.json {
        let total = results.len();
        eprintln!("\nRendered {total} files ({fail_count} failures)");
    }

    if fail_count > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn run_stdin_mode(cli: &Cli) -> ExitCode {
    let Some(output) = cli.output.clone() else {
        eprintln!("error: --output is required when reading from stdin");
        return ExitCode::from(1);
    };

    if cli.dry_run {
        let source = format_dry_run("<stdin content>", cli);
        println!("{source}");
        return ExitCode::SUCCESS;
    }

    let result = render_stdin(&output, cli);

    if cli.json {
        result.print_json();
    } else {
        result.print_human();
    }

    if result.success {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
