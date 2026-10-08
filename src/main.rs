//! CodeGuards CLI entrypoint and subcommand dispatch.

#![forbid(unsafe_code)]

use codeguards_mcp::cli::{run_check, run_exception, run_hook, run_init, run_serve, run_validate};
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    init_tracing();

    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "check".to_string());

    match command.as_str() {
        "menu" => run_menu_cli().await,
        "init" => run_init(args),
        "check" => run_check(args),
        "hook" => run_hook(args),
        "exception" => run_exception(args),
        "validate" => run_validate(args),
        "serve" => run_serve(args).await,
        "--version" | "-v" | "-V" => {
            println!("slugrails {} (Rust 2024 Edition)", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        "--help" | "-h" | "help" => {
            print_help();
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("Unknown command: {other}. Use 'slugrails --help' for usage.");
            ExitCode::from(2)
        }
    }
}

async fn run_menu_cli() -> ExitCode {
    match codeguards_mcp::run_menu() {
        Ok(true) => run_serve(std::iter::empty()).await,
        Ok(false) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Menu error: {e}");
            ExitCode::from(1)
        }
    }
}

fn print_help() {
    println!(
        "SlugRails MCP v{} (Rust 2024)\n\
         Architectural Guardrails & Code Governance for AI Coding Agents\n\n\
         USAGE:\n\
           slugrails menu                          Launch interactive setup & management menu\n\
           slugrails init [--root <path>]          Scaffold starter ARCHITECTURE.md with slugplan\n\
           slugrails check [--all] [--root <path>]  Run rail checks on modified files (or all)\n\
           slugrails hook [install|uninstall]      Manage git pre-commit hook\n\
           slugrails validate [--root <path>]      Validate .planning/ARCHITECTURE.md against tests\n\
           slugrails exception add <file> <guard> --reason=\"...\"\n\
           slugrails exception list\n\
           slugrails exception revoke <token>\n\
           slugrails serve [--port <port>]         Start MCP server (stdio default)",
        env!("CARGO_PKG_VERSION")
    );
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}
