//! CLI `validate` subcommand handler.

use super::args::extract_root;
use crate::contract::validate_architecture;
use crate::library::ensure_test_library_seeded;
use std::process::ExitCode;

/// Executes the `slugrails validate` command.
pub fn run_validate(args: impl Iterator<Item = String>) -> ExitCode {
    let args_vec: Vec<String> = args.collect();
    if super::args::has_help_flag(&args_vec) {
        println!(
            "Usage: slugrails validate [--root <path>]\n\nValidates .planning/ARCHITECTURE.md against tests and source disk reality."
        );
        return ExitCode::SUCCESS;
    }
    let root = extract_root(&args_vec);

    let catalog = match ensure_test_library_seeded() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[SLUGRAILS-ERROR] Failed to seed test library: {e}");
            return ExitCode::from(2);
        }
    };

    let result = match validate_architecture(&root, &catalog) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[SLUGRAILS-ERROR] Validation error: {e}");
            return ExitCode::from(2);
        }
    };

    if result.is_valid {
        println!(
            "[SLUGRAILS-PASS] .planning/ARCHITECTURE.md is valid and all required guards exist!"
        );
        println!("  Active Guards: {:?}", result.ready_guards);
        ExitCode::SUCCESS
    } else {
        eprintln!("[SLUGRAILS-BLOCKED] Architecture validation failed:");
        for err in result.errors {
            eprintln!("  ❌ {err}");
        }
        ExitCode::from(1)
    }
}
