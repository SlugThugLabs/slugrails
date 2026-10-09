//! `SlugRails` CLI entrypoint and subcommand dispatch.

#![forbid(unsafe_code)]

use slugrails::cli::{
    run_check, run_exception, run_fix, run_hook, run_init, run_plan, run_scaffold_tests, run_serve,
    run_validate,
};
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    init_tracing();

    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "check".to_string());

    match command.as_str() {
        "menu" => run_menu_cli().await,
        "init" => run_init(args),
        "plan" => run_plan(args),
        "scaffold-tests" | "harness" => run_scaffold_tests(args),
        "check" => run_check(args),
        "fix" => run_fix(args),
        "hook" => run_hook(args),
        "exception" => run_exception(args),
        "validate" => run_validate(args),
        "serve" => run_serve(args).await,

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
        other => {
            eprintln!("Unknown command: {other}. Use 'slugrails --help' for usage.");
            ExitCode::from(2)
        }
    }
}

async fn run_menu_cli() -> ExitCode {
    match slugrails::run_menu() {
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

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));

    if let Err(e) = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init()
    {
        eprintln!("Warning: failed to initialize tracing: {e}");
    }
}
