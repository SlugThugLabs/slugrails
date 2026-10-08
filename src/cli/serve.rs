//! CLI `serve` subcommand handler (starts stdio MCP server).

use codeguards_mcp::library::ensure_test_library_seeded;
use codeguards_mcp::server::CodeGuardsMcpServer;
use rmcp::service::ServiceExt;
use rmcp::transport::io::stdio;
use std::process::ExitCode;

/// Executes the `codeguards-mcp serve` command.
pub async fn run_serve(_args: impl Iterator<Item = String>) -> ExitCode {
    let catalog = match ensure_test_library_seeded() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[CODEGUARD-ERROR] Failed to seed test library: {e}");
            return ExitCode::from(2);
        }
    };

    let handler = CodeGuardsMcpServer::new(catalog);

    match handler.serve(stdio()).await {
        Ok(service) => match service.waiting().await {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("[CODEGUARD-ERROR] Service error: {e}");
                ExitCode::from(1)
            }
        },
        Err(e) => {
            eprintln!("[CODEGUARD-ERROR] Server start error: {e}");
            ExitCode::from(1)
        }
    }
}
