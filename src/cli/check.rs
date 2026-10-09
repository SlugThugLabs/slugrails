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

use std::path::Path;
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
        print_check_help();
        return ExitCode::SUCCESS;
    }

    if let Err(err) = validate_known_flags(
        &args_vec,
        &[
            "--all", "-a", "--diff", "-d", "--root", "-r", "--format", "-f", "--help", "-h",
            "--fix",
        ],
    ) {
        eprintln!("{err}");
        return ExitCode::from(2);
    }

    let root = extract_root(&args_vec);
    let check_diff = has_flag(&args_vec, &["--diff", "-d"]);
    let should_fix = has_flag(&args_vec, &["--fix"]);
    let format = parse_format(
        &extract_option(&args_vec, "--format", Some("-f")).unwrap_or_else(|| "text".to_string()),
    );

    match execute_check_pipeline(&root, check_diff, should_fix) {
        Ok(Some(report)) => {
            print_report(format, &report);
            if report.is_pass() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Ok(None) => {
            print_empty_report(format);
            ExitCode::SUCCESS
        }
        Err(code) => code,
    }
}

fn execute_check_pipeline(
    root: &Path,
    check_diff: bool,
    should_fix: bool,
) -> Result<Option<GuardReport>, ExitCode> {
    let catalog = ensure_test_library_seeded().map_err(|e| {
        tracing::error!(error = %e, "Failed to seed test library");
        eprintln!("[SLUGRAILS-ERROR] Failed to seed test library: {e}");
        ExitCode::from(2)
    })?;

    let contract = load_architecture(root).map_err(|e| {
        tracing::error!(error = %e, path = %root.display(), "Could not load ARCHITECTURE.md");
        eprintln!("[SLUGRAILS-ERROR] Could not load ARCHITECTURE.md: {e}");
        ExitCode::from(2)
    })?;

    let exceptions = ProjectExceptions::load(root).map_err(|e| {
        tracing::error!(error = %e, path = %root.display(), "Failed to load exception store");
        eprintln!("[SLUGRAILS-ERROR] Failed to load exception store: {e}");
        ExitCode::from(2)
    })?;
    let collection = if check_diff {
        collect_git_diff_files(root)
    } else {
        collect_source_files(root)
    }
    .map_err(|e| {
        tracing::error!(error = %e, path = %root.display(), "Failed to collect files for checking");
        eprintln!("[SLUGRAILS-ERROR] Failed to collect files: {e}");
        ExitCode::from(2)
    })?;
    let files = &collection.files;

    if files.is_empty() {
        return Ok(None);
    }

    let mut report =
        run_guard_checks(root, files, &contract, &catalog, &exceptions).map_err(|e| {
            eprintln!("[SLUGRAILS-ERROR] Check evaluation failed: {e}");
            ExitCode::from(2)
        })?;

    if should_fix && !report.violations.is_empty() {
        let fixed = auto_remediate_violations(root, &report.violations);
        if fixed > 0 {
            eprintln!("[SLUGRAILS-FIX] Remediated {fixed} violation(s). Re-evaluating...");
            if let Ok(rerun) = run_guard_checks(root, files, &contract, &catalog, &exceptions) {
                report = rerun;
            }
        }
    }

    report.truncated = collection.truncated;
    report.unreadable_entries = collection.unreadable_entries;

    Ok(Some(report))
}

fn print_check_help() {
    println!(
        "Usage: slugrails check [OPTIONS]\n\n\
         Enforces architectural invariants and guardrails.\n\n\
         Options:\n\
           --diff, -d           Scan git modified files only (fast-path)\n\
           --all, -a            Scan all source files (default)\n\
           --fix                Automatically remediate auto-fixable violations\n\
           --format, -f <FMT>   Output format: 'text', 'json', or 'sarif' (default: text)\n\
           --root, -r <PATH>    Path to project root (default: current dir)\n\
           --help, -h           Show this help message"
    );
}

fn print_empty_report(format: OutputFormat) {
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
