//! CLI `serve` subcommand handler (starts stdio or TCP MCP server).

use crate::library::ensure_test_library_seeded;
use crate::server::SlugRailsMcpServer;
use rmcp::service::ServiceExt;
use rmcp::transport::io::stdio;
use std::process::ExitCode;

/// Executes the `slugrails serve` command.
pub async fn run_serve(args: impl Iterator<Item = String>) -> ExitCode {
    let args_vec: Vec<String> = args.collect();
    if super::args::has_help_flag(&args_vec) {
        println!(
            "Usage: slugrails serve [--port <number>]\n\n\
             Starts the MCP server. Defaults to stdio transport, or TCP socket if --port is provided."
        );
        return ExitCode::SUCCESS;
    }

    let catalog = match ensure_test_library_seeded() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[SLUGRAILS-ERROR] Failed to seed test library: {e}");
            return ExitCode::from(2);
        }
    };

    let handler = SlugRailsMcpServer::new(catalog);

    if let Some(port_str) = super::args::extract_option(&args_vec, "--port", Some("-p")) {
        let port: u16 = match port_str.parse() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("[SLUGRAILS-ERROR] Invalid port number: '{port_str}'");
                return ExitCode::from(2);
            }
        };
        return serve_tcp(handler, port).await;
    }

    serve_stdio(handler).await
}

async fn serve_stdio(handler: SlugRailsMcpServer) -> ExitCode {
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

async fn serve_tcp(handler: SlugRailsMcpServer, port: u16) -> ExitCode {
    let addr = format!("127.0.0.1:{port}");
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => {
            eprintln!("[SLUGRAILS] TCP MCP server listening on {addr}");
            l
        }
        Err(e) => {
            eprintln!("[SLUGRAILS-ERROR] Failed to bind TCP listener on {addr}: {e}");
            return ExitCode::from(1);
        }
    };

    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                tracing::info!("Accepted MCP connection from {peer}");
                let srv = handler.clone();
                tokio::spawn(async move {
                    if let Ok(svc) = srv.serve(stream).await {
                        let _ = svc.waiting().await;
                    }
                });
            }
            Err(e) => {
                eprintln!("[SLUGRAILS-ERROR] TCP accept error: {e}");
                break;
            }
        }
    }
    ExitCode::SUCCESS
}
