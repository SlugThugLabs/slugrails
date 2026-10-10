//! `SlugRails` CLI entrypoint and subcommand dispatch.

#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "check".to_string());

    match command.as_str() {
        "--version" | "-v" | "-V" => {
            println!(
                "slugrails {} (Rust 2024 Edition)",
                env!("CARGO_PKG_VERSION")
            );
            ExitCode::SUCCESS
        }
        "--help" | "-h" | "help" => {
            print_help();
            ExitCode::SUCCESS
        }
        "menu" | "init" | "plan" | "scaffold-tests" | "harness" | "check" | "fix" | "hook"
        | "exception" | "validate" | "serve" => {
            println!(
                "slugrails {command}: foundation milestone M1 active; core command pending M2/M3"
            );
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("Unknown command: {other}. Use 'slugrails --help' for usage.");
            ExitCode::from(2)
        }
    }
}

fn print_help() {
    println!(
        "SlugRails v{} (Rust 2024)\n\
         Architectural Rail Builder & Contract Scaffolding for Native Toolchains & Agents\n\n\
         USAGE:\n\
           slugrails menu                          Launch interactive setup & management menu\n\
           slugrails init [--root <path>]          Scaffold starter ARCHITECTURE.md with slugplan\n\
           slugrails plan [--root <path>]          Converse/discover terms & auto-scaffold architecture + tests\n\
           slugrails scaffold-tests [--root <path>] Generate tests/slugrails.rs integration test harness\n\
           slugrails check [--all] [--root <path>]  Run rail checks on modified files (or all)\n\
           slugrails fix [--root <path>]            Remediate auto-fixable violations\n\
           slugrails hook [install|uninstall]      Manage git pre-commit hook\n\
           slugrails validate [--root <path>]      Validate .planning/ARCHITECTURE.md against tests\n\
           slugrails exception add <file> <guard> --reason=\"...\"\n\
           slugrails exception list\n\
           slugrails exception revoke <token>\n\
           slugrails serve [--port <num>]          Start MCP server (stdio default, or TCP with --port)",
        env!("CARGO_PKG_VERSION")
    );
}
