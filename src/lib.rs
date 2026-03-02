mod cli;
mod render;
mod report;

use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};

use clap::Parser;
use rayon::prelude::*;

use cli::Cli;
use render::{
    check_dependencies, default_output_path, format_dry_run_command, render_one, render_stdin,
};

#[must_use]
pub fn run() -> ExitCode {
    let cli = Cli::parse();
    run_with(&cli)
}

#[must_use]
pub fn run_with(cli: &Cli) -> ExitCode {
    // Check dependencies unless dry-run
    if !cli.dry_run
        && let Err(missing) = check_dependencies()
    {
        eprintln!(
            "error: missing required dependencies: {}",
            missing.join(", ")
        );
        eprintln!("install them and ensure they are on PATH");
        return ExitCode::from(2);
    }

    // Stdin mode: no files given
    if cli.files.is_empty() {
        return run_stdin(cli);
    }

    // Validate: -o only valid with single file
    if cli.output.is_some() && cli.files.len() > 1 {
        eprintln!("error: --output can only be used with a single input file");
        return ExitCode::from(1);
    }

    // Dry-run mode
    if cli.dry_run {
        for file in &cli.files {
            let output = cli
                .output
                .clone()
                .unwrap_or_else(|| default_output_path(file));
            let cmd = format_dry_run_command(&file.display().to_string(), &output, cli);
            println!("{cmd}");
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

    // Multi-file parallel
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

    let any_failed = AtomicBool::new(false);

    pool.install(|| {
        cli.files.par_iter().for_each(|input| {
            let output = default_output_path(input);
            let result = render_one(input, &output, cli);

            if !result.success {
                any_failed.store(true, Ordering::Relaxed);
            }

            if cli.json {
                result.print_json();
            } else {
                result.print_human();
            }
        });
    });

    if !cli.json {
        let total = cli.files.len();
        let failed = if any_failed.load(Ordering::Relaxed) {
            "some"
        } else {
            "0"
        };
        eprintln!("\nRendered {total} files ({failed} failures)");
    }

    if any_failed.load(Ordering::Relaxed) {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn run_stdin(cli: &Cli) -> ExitCode {
    let Some(output) = cli.output.clone() else {
        eprintln!("error: --output is required when reading from stdin");
        return ExitCode::from(1);
    };

    if cli.dry_run {
        let cmd = format_dry_run_command("<stdin>", &output, cli);
        println!("{cmd}");
        return ExitCode::SUCCESS;
    }

    match render_stdin(&output, cli) {
        Ok(result) => {
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
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}
