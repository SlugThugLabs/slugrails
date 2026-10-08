//! CLI `check` subcommand handler.

use super::sarif::format_sarif;
use codeguards_mcp::analyzer::{collect_git_diff_files, collect_source_files};
use codeguards_mcp::contract::load_architecture;
use codeguards_mcp::guards::run_guard_checks;
use codeguards_mcp::library::ensure_test_library_seeded;
use codeguards_mcp::storage::ProjectExceptions;
use std::path::PathBuf;
use std::process::ExitCode;

/// Output formats supported by `check`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
    Sarif,
}

/// Executes the `codeguards-mcp check` command.
pub fn run_check(mut args: impl Iterator<Item = String>) -> ExitCode {
    let mut check_all = false;
    let mut root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut format = OutputFormat::Text;

    while let Some(arg) = args.next() {
        if let Some(r) = arg.strip_prefix("--root=").or_else(|| arg.strip_prefix("-r=")) {
            root = PathBuf::from(r);
        } else if (arg == "--root" || arg == "-r") && let Some(r) = args.next() {
            root = PathBuf::from(r);
        } else if let Some(f) = arg.strip_prefix("--format=").or_else(|| arg.strip_prefix("-f=")) {
            format = parse_format(f);
        } else if (arg == "--format" || arg == "-f") && let Some(f) = args.next() {
            format = parse_format(&f);
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
        match format {
            OutputFormat::Json => println!("{{\"files_checked\":0,\"violations\":[]}}"),
            OutputFormat::Sarif => {
                let empty_report = codeguards_mcp::types::GuardReport::default();
                println!("{}", serde_json::to_string_pretty(&format_sarif(&empty_report)).unwrap_or_default());
            }
            OutputFormat::Text => println!("[CODEGUARD-PASS] No source files to check."),
        }
        return ExitCode::SUCCESS;
    }

    let report = match run_guard_checks(&root, &files, &contract, &catalog, &exceptions) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[CODEGUARD-ERROR] Check evaluation failed: {e}");
            return ExitCode::from(2);
        }
    };

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

fn render_text_report(report: &codeguards_mcp::types::GuardReport) {
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
    } else {
        println!("\n════════════════════════════════════════════════════════════");
        println!(" [CODEGUARD-BLOCKED] ARCHITECTURE GUARD VIOLATIONS DETECTED");
        println!("════════════════════════════════════════════════════════════");
        for v in &report.violations {
            let line_str = v.line.map_or(String::new(), |l| format!(":{l}"));
            println!("\n❌ [{}] {}{}", v.guard_id, v.file.display(), line_str);
            println!("   Reason:      {}", v.message);
            if let Some(fix) = &v.fix_suggestion {
                println!("   Remediation: {fix}");
            }
        }
        println!("\nTotal violations: {}", report.violations.len());
    }
}
