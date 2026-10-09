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
                println!("{}", serde_json::to_string_pretty(&format_sarif(&empty)).unwrap_or_default());
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

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
        }
        OutputFormat::Sarif => {
            println!("{}", serde_json::to_string_pretty(&format_sarif(&report)).unwrap_or_default());
        }
        OutputFormat::Text => render_text_report(&report),
    }

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

fn render_text_report(report: &GuardReport) {
    if !report.unevaluated_tests.is_empty() {
        println!(
            "[SLUGRAILS-WARNING] {} enabled rule(s) were NOT evaluated (no implementation):",
            report.unevaluated_tests.len()
        );
        for rule in &report.unevaluated_tests {
            println!("   - {rule}");
        }
    }

    if report.is_pass() {
        println!(
            "[SLUGRAILS-PASS] {} files verified across {} active guard rules in {}ms.",
            report.total_files_checked,
            report.passed_tests.len(),
            report.duration_ms
        );
    } else {
        println!("\n\x1b[1;31merror\x1b[0m: architecture guard violations detected ({})", report.violations.len());
        for v in &report.violations {
            let line_num = v.line.unwrap_or(1);
            let snippet = std::fs::read_to_string(&v.file)
                .ok()
                .and_then(|c| c.lines().nth(line_num.saturating_sub(1)).map(|s| s.trim_end().to_string()))
                .unwrap_or_default();

            println!("\n  \x1b[1;31m[{}]\x1b[0m \x1b[1m{}\x1b[0m", v.guard_id, v.message);
            println!("    \x1b[36m-->\x1b[0m {}:{line_num}", v.file.display());
            println!("     \x1b[36m|\x1b[0m");
            if !snippet.is_empty() {
                println!(" {line_num:3} \x1b[36m|\x1b[0m {snippet}");
                println!("     \x1b[36m|\x1b[0m \x1b[1;31m^\x1b[0m \x1b[31mviolation detected here\x1b[0m");
            } else {
                println!(" {line_num:3} \x1b[36m|\x1b[0m \x1b[31m^^^^ violation coordinate\x1b[0m");
            }
            println!("     \x1b[36m|\x1b[0m");
            if let Some(fix) = &v.fix_suggestion {
                println!("     \x1b[32m= help:\x1b[0m {fix}");
            }
            println!("     \x1b[33m= bypass:\x1b[0m slugrails exception add {} {} --reason=\"...\"", v.file.display(), v.guard_id);
        }
        println!("\n\x1b[1;31mFailed with {} violation(s)\x1b[0m", report.violations.len());
    }
}
