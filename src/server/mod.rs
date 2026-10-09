//! MCP Server implementation for `SlugRails` and `SlugPlan` governance tools.

pub mod actions;
pub mod handlers;
mod types;

pub use types::{
    AddExceptionRequest, CheckProjectRequest, CreateGuardTestRequest, ListExceptionsRequest,
    ListExceptionsResponse, ListGuardTestsRequest, ProbeArchitectureRequest,
    RevokeExceptionRequest, RevokeExceptionResponse, ScaffoldHarnessRequest,
    ValidateArchitectureRequest,
};

use crate::library::catalog::GuardCatalog;
use rmcp::handler::server::ServerHandler;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{Implementation, ProtocolVersion, ServerCapabilities, ServerConfig};
use rmcp::{ErrorData, Json, tool, tool_handler, tool_router};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;

const INSTRUCTIONS: &str = "SlugRails: Architectural Rail Builder. Use slugplan_probe to inspect projects, interview users in chat to discover domain terms, and scaffold contracts with tests/slugrails.rs harnesses for native cargo test enforcement.";

#[derive(Clone)]
pub struct SlugRailsMcpServer {
    pub catalog: Arc<Mutex<GuardCatalog>>,
    tool_router: rmcp::handler::server::tool::ToolRouter<Self>,
}

impl SlugRailsMcpServer {
    #[must_use]
    pub fn new(catalog: GuardCatalog) -> Self {
        Self {
            catalog: Arc::new(Mutex::new(catalog)),
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl SlugRailsMcpServer {
    #[tool(
        description = "Validates .planning/ARCHITECTURE.md contract against disk reality and guard catalog."
    )]
    async fn validate_architecture(
        &self,
        request: Parameters<ValidateArchitectureRequest>,
    ) -> Result<Json<crate::contract::ValidationResult>, ErrorData> {
        handlers::handle_validate_architecture(&self.catalog, request.0).await
    }

    #[tool(
        description = "Checks project source files against all active architectural guardrails and anti-drift rules."
    )]
    async fn check_project(
        &self,
        request: Parameters<CheckProjectRequest>,
    ) -> Result<Json<crate::types::GuardReport>, ErrorData> {
        handlers::handle_check_project(&self.catalog, request.0).await
    }

    #[tool(
        description = "SlugPlan architectural probe: interviews intent and produces recommended layers and guard contracts."
    )]
    async fn slugplan_probe(
        &self,
        request: Parameters<ProbeArchitectureRequest>,
    ) -> Result<Json<serde_json::Value>, ErrorData> {
        handlers::handle_probe(request.0).await
    }

    #[tool(
        description = "Lists all modular guard tests available in ~/.slugthuglabs/slugrails/tests/."
    )]
    async fn list_guard_tests(
        &self,
        request: Parameters<ListGuardTestsRequest>,
    ) -> Result<Json<BTreeMap<String, crate::library::catalog::GuardCatalogEntry>>, ErrorData> {
        let category_filter = request.0.category.as_deref();
        let catalog = self.catalog.lock().await;

        let filtered: BTreeMap<String, _> = catalog
            .tests
            .iter()
            .filter(|(_, entry)| category_filter.is_none_or(|cat| entry.category.starts_with(cat)))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Ok(Json(filtered))
    }

    #[tool(
        description = "Creates a new modular guard test definition in ~/.slugthuglabs/slugrails/tests/."
    )]
    async fn create_guard_test(
        &self,
        request: Parameters<CreateGuardTestRequest>,
    ) -> Result<Json<String>, ErrorData> {
        handlers::handle_create_guard_test(&self.catalog, request.0).await
    }

    #[tool(
        description = "Authorizes a user-approved exception token in .planning/slugrails/exceptions.json."
    )]
    async fn add_exception(
        &self,
        request: Parameters<AddExceptionRequest>,
    ) -> Result<Json<crate::types::ExceptionEntry>, ErrorData> {
        handlers::handle_add_exception(request.0)
    }

    #[tool(
        description = "Generates tests/slugrails.rs integration test harness for native cargo test enforcement."
    )]
    async fn scaffold_test_harness(
        &self,
        request: Parameters<ScaffoldHarnessRequest>,
    ) -> Result<Json<String>, ErrorData> {
        handlers::handle_scaffold_harness(request.0)
    }

    #[tool(
        description = "Lists all active architectural exceptions in .planning/slugrails/exceptions.json."
    )]
    async fn list_exceptions(
        &self,
        request: Parameters<ListExceptionsRequest>,
    ) -> Result<Json<ListExceptionsResponse>, ErrorData> {
        handlers::handle_list_exceptions(request.0)
    }

    #[tool(
        description = "Revokes an authorized exception token in .planning/slugrails/exceptions.json."
    )]
    async fn revoke_exception(
        &self,
        request: Parameters<RevokeExceptionRequest>,
    ) -> Result<Json<RevokeExceptionResponse>, ErrorData> {
        handlers::handle_revoke_exception(request.0)
    }
}

#[allow(clippy::unused_async_trait_impl)]
#[tool_handler(router = self.tool_router)]
impl ServerHandler for SlugRailsMcpServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::LATEST)
            .with_instructions(INSTRUCTIONS)
            .with_server_info(Implementation::new(
                "slugrails-mcp",
                env!("CARGO_PKG_VERSION"),
            ))
    }
}
