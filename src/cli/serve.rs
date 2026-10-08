//! CLI `serve` subcommand handler (starts stdio MCP server).

use crate::library::ensure_test_library_seeded;
use crate::server::CodeGuardsMcpServer;
use rmcp::service::ServiceExt;
use rmcp::transport::io::stdio;
use std::process::ExitCode;

/// Executes the `codeguards-mcp serve` command.
pub async fn run_serve(args: impl Iterator<Item = String>) -> ExitCode {
    let args_vec: Vec<String> = args.collect();
    if args_vec.iter().any(|a| a == "--port" || a.starts_with("--port=")) {
        eprintln!("[SLUGRAILS-NOTICE] TCP transport (--port) reserved for future release. Using stdio transport.");
    }

    let catalog = match ensure_test_library_seeded() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[CODEGUARD-ERROR] Failed to seed test library: {e}");
            return ExitCode::from(2);
        }
    };

    let handler = CodeGuardsMcpServer::new(catalog);
    tracing::info!("Starting SlugRails stdio MCP server");

    match handler.serve(stdio()).await {
        Ok(service) => match service.waiting().await {
            Ok(_) => {
                tracing::info!("SlugRails stdio MCP server terminated gracefully");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("[SLUGRAILS-ERROR] Service error: {e}");
                ExitCode::from(1)
            }
        },
        Err(e) => {
            eprintln!("[SLUGRAILS-ERROR] Server start error: {e}");
            ExitCode::from(1)
        }
    }
}
