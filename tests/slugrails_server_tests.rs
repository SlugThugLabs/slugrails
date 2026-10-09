//! Integration tests for SlugRails MCP server tools.

use slugrails::library::catalog::GuardCatalog;
use slugrails::server::SlugRailsMcpServer;
use rmcp::handler::server::ServerHandler;

#[test]
fn test_slugrails_mcp_server_info() {
    let catalog = GuardCatalog::default();
    let server = SlugRailsMcpServer::new(catalog);
    let info = server.get_info();

    assert_eq!(info.server_info.name, "slugrails-mcp");
    assert!(info.instructions.unwrap_or_default().contains("SlugRails"));
}
