//! CLI `validate` subcommand handler.

use codeguards_mcp::contract::validate_architecture;
use codeguards_mcp::library::ensure_test_library_seeded;
use std::path::PathBuf;
use std::process::ExitCode;

/// Executes the `codeguards-mcp validate` command.
pub fn run_validate(mut args: impl Iterator<Item = String>) -> ExitCode {
    let mut root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    while let Some(arg) = args.next() {
        if arg == "--root"
            && let Some(r) = args.next()
        {
            root = PathBuf::from(r);
        }
    }

    let catalog = match ensure_test_library_seeded() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[CODEGUARD-ERROR] Failed to seed test library: {e}");
            return ExitCode::from(2);
        }
    };

    let result = match validate_architecture(&root, &catalog) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[CODEGUARD-ERROR] Validation error: {e}");
            return ExitCode::from(2);
        }
    };

    if result.is_valid {
        println!(
            "[CODEGUARD-PASS] .planning/ARCHITECTURE.md is valid and all required guards exist!"
        );
        println!("  Active Guards: {:?}", result.ready_guards);
        ExitCode::SUCCESS
    } else {
        eprintln!("[CODEGUARD-BLOCKED] Architecture validation failed:");
        for err in result.errors {
            eprintln!("  ❌ {err}");
        }
        ExitCode::from(1)
    }
}
