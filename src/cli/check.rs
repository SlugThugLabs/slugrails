//! CLI `check` subcommand handler.

use super::args::{extract_option, extract_root, has_flag, validate_known_flags};
use super::fix::auto_remediate_violations;
use super::sarif::format_sarif;
use crate::analyzer::{collect_git_diff_files, collect_source_files};
use crate::contract::load_architecture;
use crate::guards::run_guard_checks;
use crate::library::ensure_test_library_seeded;
use crate::storage::ProjectExceptions;
use crate::types::GuardReport;

use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
    Sarif,
}

/// Executes the `slugrails check` command.
pub fn run_check(args: impl Iterator<Item = String>) -> ExitCode {
    let args_vec: Vec<String> = args.collect();
    if crate::cli::args::has_help_flag(&args_vec) {
        println!(
            "Usage: slugrails check [OPTIONS]\n\n\
             Enforces architectural invariants and guardrails.\n\n\
             Options:\n\
               --all, -a            Scan all source files instead of git modified files\n\
               --fix                Automatically remediate auto-fixable violations\n\
               --format, -f <FMT>   Output format: 'text', 'json', or 'sarif' (default: text)\n\
               --root, -r <PATH>    Path to project root (default: current dir)\n\
               --help, -h           Show this help message"
        );
        return ExitCode::SUCCESS;
    }

    if let Err(err) = validate_known_flags(
        &args_vec,
        &[
            "--all", "-a", "--root", "-r", "--format", "-f", "--help", "-h", "--fix",
        ],
    ) {
        eprintln!("{err}");
        return ExitCode::from(2);
    }

    let root = extract_root(&args_vec);
    let check_all = has_flag(&args_vec, &["--all", "-a"]);
    let should_fix = has_flag(&args_vec, &["--fix"]);
    let format = parse_format(
        &extract_option(&args_vec, "--format", Some("-f")).unwrap_or_else(|| "text".to_string()),
    );

    let catalog = match ensure_test_library_seeded() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "Failed to seed test library");
            eprintln!("[SLUGRAILS-ERROR] Failed to seed test library: {e}");
            return ExitCode::from(2);
        }
    };

    let contract = match load_architecture(&root) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, path = %root.display(), "Could not load ARCHITECTURE.md");
            eprintln!("[SLUGRAILS-ERROR] Could not load ARCHITECTURE.md: {e}");
            return ExitCode::from(2);
        }
    };

    let exceptions = ProjectExceptions::load(&root).unwrap_or_default();
    let files = if check_all {
        collect_source_files(&root).unwrap_or_default()
    } else {
        collect_git_diff_files(&root).unwrap_or_default()
    };

    if files.is_empty() {
        match format {
            OutputFormat::Json => println!("{{\"files_checked\":0,\"violations\":[]}}"),
            OutputFormat::Sarif => {
                let empty = GuardReport::default();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&format_sarif(&empty)).unwrap_or_default()
                );
            }
            OutputFormat::Text => println!("[SLUGRAILS-PASS] No source files to check."),
        }
        return ExitCode::SUCCESS;
    }

    let mut report = match run_guard_checks(&root, &files, &contract, &catalog, &exceptions) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[SLUGRAILS-ERROR] Check evaluation failed: {e}");
            return ExitCode::from(2);
        }
    };

    if should_fix && !report.violations.is_empty() {
        let fixed = auto_remediate_violations(&root, &report.violations);
        if fixed > 0 {
            println!("[SLUGRAILS-FIX] Remediated {fixed} violation(s). Re-evaluating...");
            if let Ok(rerun) = run_guard_checks(&root, &files, &contract, &catalog, &exceptions) {
                report = rerun;
            }
        }
    }

    print_report(format, &report);

    if report.is_pass() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn parse_format(f: &str) -> OutputFormat {
    match f.to_ascii_lowercase().as_str() {
        "json" => OutputFormat::Json,
        "sarif" => OutputFormat::Sarif,
        _ => OutputFormat::Text,
    }
}

fn print_report(format: OutputFormat, report: &GuardReport) {
    match format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(report).unwrap_or_default()
            );
        }
        OutputFormat::Sarif => {
            println!(
                "{}",
                serde_json::to_string_pretty(&format_sarif(report)).unwrap_or_default()
            );
        }
        OutputFormat::Text => crate::cli::format::print_text_report(report),
    }
}
