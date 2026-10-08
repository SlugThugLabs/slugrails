//! CodeGuards CLI entrypoint and subcommand dispatch.

#![forbid(unsafe_code)]

mod cli;

use cli::{run_check, run_exception, run_serve, run_validate};
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "check".to_string());

    match command.as_str() {
        "menu" => run_menu_cli().await,
        "check" => run_check(args),
        "exception" => run_exception(args),
        "validate" => run_validate(args),
        "serve" => run_serve(args).await,
        "--help" | "-h" | "help" => {
            print_help();
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("Unknown command: {other}. Use 'codeguards-mcp --help' for usage.");
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
        "CodeGuards MCP (Rust 2024)\n\
         Continuous Code Governance & Architectural Enforcement\n\n\
         USAGE:\n\
           codeguards-mcp menu                          Launch interactive setup & management menu\n\
           codeguards-mcp check [--all] [--root <path>]  Run guard checks on modified files (or all)\n\
           codeguards-mcp validate [--root <path>]      Validate .planning/ARCHITECTURE.md against tests\n\
           codeguards-mcp exception add <file> <guard> --reason=\"...\"\n\
           codeguards-mcp exception list\n\
           codeguards-mcp exception revoke <token>\n\
           codeguards-mcp serve [--port <port>]         Start MCP server (stdio default)"
    );
}
