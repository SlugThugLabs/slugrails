//! Auto-remediation engine for self-healing guard violations.

use super::args::{extract_root, has_flag, validate_known_flags};
use crate::analyzer::{collect_git_diff_files, collect_source_files};
use crate::contract::load_architecture;
use crate::guards::run_guard_checks;
use crate::library::ensure_test_library_seeded;
use crate::storage::ProjectExceptions;
use crate::types::Violation;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

/// Entry point for `slugrails fix` subcommand.
pub fn run_fix(args: impl Iterator<Item = String>) -> ExitCode {
    let args_vec: Vec<String> = args.collect();
    if crate::cli::args::has_help_flag(&args_vec) {
        println!(
            "Usage: slugrails fix [OPTIONS]\n\n\
             Automatically remediates self-healing guard violations.\n\n\
             Options:\n\
               --diff, -d           Remediate git modified files only (fast-path)\n\
               --all, -a            Remediate all source files (default)\n\
               --root, -r <PATH>    Path to project root (default: current dir)\n\
               --help, -h           Show this help message"
        );
        return ExitCode::SUCCESS;
    }

    if let Err(err) = validate_known_flags(
        &args_vec,
        &[
            "--all", "-a", "--diff", "-d", "--root", "-r", "--help", "-h",
        ],
    ) {
        eprintln!("{err}");
        return ExitCode::from(2);
    }

    let root = extract_root(&args_vec);
    let check_diff = has_flag(&args_vec, &["--diff", "-d"]);
    let catalog = match ensure_test_library_seeded() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[SLUGRAILS-ERROR] Failed to seed test library: {e}");
            return ExitCode::from(2);
        }
    };

    let contract = match load_architecture(&root) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[SLUGRAILS-ERROR] Could not load ARCHITECTURE.md: {e}");
            return ExitCode::from(2);
        }
    };

    let exceptions = match ProjectExceptions::load(&root) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[SLUGRAILS-ERROR] Failed to load exception store: {e}");
            return ExitCode::from(2);
        }
    };
    let collection = match if check_diff {
        collect_git_diff_files(&root)
    } else {
        collect_source_files(&root)
    } {
        Ok(f) => f,
        Err(e) => {
            tracing::error!(error = %e, path = %root.display(), "Failed to collect files for remediation");
            eprintln!("[SLUGRAILS-ERROR] Failed to collect files: {e}");
            return ExitCode::from(2);
        }
    };

    let files = &collection.files;
    if files.is_empty() {
        println!("[SLUGRAILS-PASS] No violations found to remediate.");
        return ExitCode::SUCCESS;
    }

    let mut report = match run_guard_checks(&root, files, &contract, &catalog, &exceptions) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[SLUGRAILS-ERROR] Check evaluation failed: {e}");
            return ExitCode::from(2);
        }
    };

    report.truncated = collection.truncated;
    report.unreadable_entries = collection.unreadable_entries;

    if report.violations.is_empty() {
        println!("[SLUGRAILS-PASS] No violations found to remediate.");
        return ExitCode::SUCCESS;
    }

    let fixed = auto_remediate_violations(&root, &report.violations);
    println!("[SLUGRAILS-FIX] Successfully remediated {fixed} violation(s).");
    ExitCode::SUCCESS
}

/// Automatically remediates supported violations (e.g. hygiene/no-debug-prints).
///
/// Returns the number of fixed violations.
#[must_use]
pub fn auto_remediate_violations(project_root: &Path, violations: &[Violation]) -> usize {
    let mut fixed_count = 0;
    let mut affected_files: HashSet<&Path> = HashSet::new();

    for v in violations {
        if v.guard_id == "hygiene/no-debug-prints" {
            affected_files.insert(&v.file);
        }
    }

    for rel_file in affected_files {
        let full_path = if rel_file.is_absolute() {
            rel_file.to_path_buf()
        } else {
            project_root.join(rel_file)
        };

        if let Ok(content) = fs::read_to_string(&full_path) {
            let mut modified = false;
            let mut new_lines = Vec::new();

            for line in content.lines() {
                let trimmed = line.trim();
                // Strip standalone debug print statements
                if (trimmed.starts_with("dbg!(") && trimmed.ends_with(");"))
                    || (trimmed.starts_with("console.log(") && trimmed.ends_with(");"))
                {
                    modified = true;
                    fixed_count += 1;
                } else {
                    new_lines.push(line);
                }
            }

            if modified {
                let mut out = new_lines.join("\n");
                if content.ends_with('\n') {
                    out.push('\n');
                }
                if fs::write(&full_path, out).is_ok() {
                    eprintln!(
                        "[SLUGRAILS-FIX] Cleaned debug statements in {}",
                        rel_file.display()
                    );
                }
            }
        }
    }

    fixed_count
}
