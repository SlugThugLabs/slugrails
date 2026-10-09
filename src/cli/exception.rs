//! CLI `exception` subcommand handler.

use crate::storage::ProjectExceptions;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Executes the `slugrails exception` command.
pub fn run_exception(mut args: impl Iterator<Item = String>) -> ExitCode {
    let sub = args.next().unwrap_or_else(|| "list".to_string());
    if sub == "--help" || sub == "-h" || sub == "help" {
        println!("Usage: slugrails exception [add|list|revoke] [OPTIONS]\n\nCommands:\n  add <file> <guard> --reason=\"...\"   Authorizes a cryptographic exception\n  list                                Lists all active exceptions\n  revoke <token>                      Revokes an authorized exception");
        return ExitCode::SUCCESS;
    }
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut exceptions = ProjectExceptions::load(&root).unwrap_or_default();

    match sub.as_str() {
        "add" => {
            let Some(file_str) = args.next() else {
                eprintln!("Usage: slugrails exception add <file> <guard> --reason=\"...\"");
                return ExitCode::from(2);
            };
            let Some(guard) = args.next() else {
                eprintln!("Usage: slugrails exception add <file> <guard> --reason=\"...\"");
                return ExitCode::from(2);
            };
            let args_vec: Vec<String> = args.collect();
            let reason = super::args::extract_option(&args_vec, "--reason", None)
                .map(|r| r.trim_matches('"').to_string())
                .unwrap_or_else(|| "User authorized exception".to_string());

            match exceptions.add_exception(Path::new(&file_str), &guard, &reason) {
                Ok(entry) => {
                    println!("[SLUGRAILS-SUCCESS] Exception authorized.");
                    println!("  Token:   {}", entry.token);
                    println!("  File:    {}", entry.file.display());
                    println!("  Guard:   {}", entry.guard_id);
                    println!(
                        "  Header:  // slugrails-exception: token={}; guard={}; reason=\"{}\"",
                        entry.token, entry.guard_id, entry.reason
                    );
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("[SLUGRAILS-ERROR] Could not save exception: {e}");
                    ExitCode::from(1)
                }
            }
        }
        "list" => {
            println!("Active exceptions for {}:", root.display());
            if exceptions.exceptions.is_empty() {
                println!("  (No active exceptions)");
            } else {
                for e in &exceptions.exceptions {
                    println!(
                        "  - [{}] {} (Guard: {}) - Reason: {}",
                        e.token,
                        e.file.display(),
                        e.guard_id,
                        e.reason
                    );
                }
            }
            ExitCode::SUCCESS
        }
        "revoke" => {
            let Some(token) = args.next() else {
                eprintln!("Usage: slugrails exception revoke <token>");
                return ExitCode::from(2);
            };
            match exceptions.revoke(&token) {
                Ok(true) => {
                    println!("[SLUGRAILS-SUCCESS] Exception token {token} revoked.");
                    ExitCode::SUCCESS
                }
                Ok(false) => {
                    eprintln!("[SLUGRAILS-ERROR] Token {token} not found.");
                    ExitCode::from(1)
                }
                Err(e) => {
                    eprintln!("[SLUGRAILS-ERROR] Revocation failed: {e}");
                    ExitCode::from(1)
                }
            }
        }
        other => {
            eprintln!("Unknown exception subcommand: {other}");
            ExitCode::from(2)
        }
    }
}
