//! CLI `check` subcommand handler.

use codeguards_mcp::analyzer::{collect_git_diff_files, collect_source_files};
use codeguards_mcp::contract::load_architecture;
use codeguards_mcp::guards::run_guard_checks;
use codeguards_mcp::library::ensure_test_library_seeded;
use codeguards_mcp::storage::ProjectExceptions;
use std::path::PathBuf;
use std::process::ExitCode;

/// Executes the `codeguards-mcp check` command.
pub fn run_check(mut args: impl Iterator<Item = String>) -> ExitCode {
    let mut check_all = false;
    let mut root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    while let Some(arg) = args.next() {
        if let Some(r) = arg.strip_prefix("--root=").or_else(|| arg.strip_prefix("-r=")) {
            root = PathBuf::from(r);
        } else if (arg == "--root" || arg == "-r") && let Some(r) = args.next() {
            root = PathBuf::from(r);
        } else if arg == "--all" || arg == "-a" {
            check_all = true;
        } else if arg == "--diff" || arg == "-d" {
            check_all = false;
        }
    }

    let catalog = match ensure_test_library_seeded() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[CODEGUARD-ERROR] Failed to seed test library: {e}");
            return ExitCode::from(2);
        }
    };

    let contract = match load_architecture(&root) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[CODEGUARD-ERROR] Could not load ARCHITECTURE.md: {e}");
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
        println!("[CODEGUARD-PASS] No source files to check.");
        return ExitCode::SUCCESS;
    }

    let report = match run_guard_checks(&root, &files, &contract, &catalog, &exceptions) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[CODEGUARD-ERROR] Check evaluation failed: {e}");
            return ExitCode::from(2);
        }
    };

    if !report.unevaluated_tests.is_empty() {
        println!(
            "[CODEGUARD-WARNING] {} enabled rule(s) were NOT evaluated (no implementation):",
            report.unevaluated_tests.len()
        );
        for rule in &report.unevaluated_tests {
            println!("   - {rule}");
        }
    }

    if report.is_pass() {
        println!(
            "[CODEGUARD-PASS] {} files verified across {} active guard rules in {}ms.",
            report.total_files_checked,
            report.passed_tests.len(),
            report.duration_ms
        );
        ExitCode::SUCCESS
    } else {
        println!("\n════════════════════════════════════════════════════════════");
        println!(" [CODEGUARD-BLOCKED] ARCHITECTURE GUARD VIOLATIONS DETECTED");
        println!("════════════════════════════════════════════════════════════");
        for v in &report.violations {
            let line_str = v.line.map_or(String::new(), |l| format!(":{l}"));
            println!("\n❌ [{}] {}{}", v.guard_id, v.file.display(), line_str);
            println!("   Reason:      {}", v.message);
            println!("   Remediation: {}", v.remediation);
        }
        println!("\nTotal violations: {}", report.violations.len());
        ExitCode::from(1)
    }
}
