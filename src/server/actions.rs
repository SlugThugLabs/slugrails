//! Scaffold, custom test creation, and exception management actions for `SlugRails` MCP.

use crate::library::catalog::GuardCatalog;
use crate::library::create_custom_guard_test;
use crate::server::handlers::{exception_store_err, resolve_safe_root};
use crate::server::types::{
    AddExceptionRequest, CreateGuardTestRequest, ListExceptionsRequest, ListExceptionsResponse,
    RevokeExceptionRequest, RevokeExceptionResponse, ScaffoldHarnessRequest,
};
use crate::storage::ProjectExceptions;
use crate::types::GuardTestDefinition;
use rmcp::{ErrorData, Json};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Handles integration test harness scaffolding.
///
/// # Errors
/// Returns [`ErrorData`] if path validation fails or scaffolding encounters an I/O error.
#[allow(clippy::needless_pass_by_value)]
pub fn handle_scaffold_harness(req: ScaffoldHarnessRequest) -> Result<Json<String>, ErrorData> {
    let project_root = resolve_safe_root(req.project_path.as_deref())?;
    match crate::contract::scaffold_tests_harness(&project_root) {
        Ok(p) => Ok(Json(format!("Scaffolded test harness: {}", p.display()))),
        Err(e) => Err(ErrorData::internal_error(
            format!("Failed to scaffold harness: {e}"),
            None,
        )),
    }
}

/// Handles creation of a custom guard test in the library catalog.
///
/// # Errors
/// Returns [`ErrorData`] if inputs fail parameter validation or file writing fails.
pub async fn handle_create_guard_test(
    catalog: &Arc<Mutex<GuardCatalog>>,
    req: CreateGuardTestRequest,
) -> Result<Json<String>, ErrorData> {
    let too_large = req.name.len() > 128
        || req.category.len() > 256
        || req.summary.len() > 512
        || req.tags.len() > 32
        || req.aliases.len() > 32
        || req.remediation.len() > 2048;
    if too_large {
        return Err(ErrorData::invalid_params(
            "Request fields exceed size limits",
            None,
        ));
    }
    let invalid_chars = req.category.contains("..")
        || req.name.contains("..")
        || req.name.contains('/')
        || req.name.contains('\\')
        || req.category.starts_with('/')
        || req.category.starts_with('\\');
    if invalid_chars {
        return Err(ErrorData::invalid_params(
            "Invalid path characters in category or name",
            None,
        ));
    }

    let def = GuardTestDefinition {
        id: format!("{}/{}", req.category, req.name),
        name: req.name,
        category: req.category,
        version: "1.0.0".to_string(),
        summary: req.summary,
        tags: req.tags,
        aliases: req.aliases,
        engine: req.engine,
        default_params: BTreeMap::new(),
        remediation: req.remediation,
    };

    let tests_root = crate::util::get_tests_dir();
    match create_custom_guard_test(&tests_root, &def, req.force) {
        Ok(path) => {
            if let Ok(new_cat) = crate::library::ensure_test_library_seeded() {
                let mut cat_lock = catalog.lock().await;
                *cat_lock = new_cat;
            }
            Ok(Json(format!(
                "Successfully created guard test at {}",
                path.display()
            )))
        }
        Err(e) => Err(ErrorData::internal_error(
            format!("Failed to create guard test: {e}"),
            None,
        )),
    }
}

/// Handles authorizing an architectural guard exception.
///
/// # Errors
///
/// Always returns [`ErrorData`]: an MCP client cannot authorize an exception.
#[allow(clippy::needless_pass_by_value)]
pub fn handle_add_exception(
    req: AddExceptionRequest,
) -> Result<Json<crate::types::ExceptionEntry>, ErrorData> {
    // The caller of an MCP tool is the agent, and it controls every field of this
    // request — including `human_confirmed`. A request field therefore cannot stand
    // in for verified human approval (audit SR-S03). Authorization happens in the
    // terminal-gated CLI instead, where a human is present.
    Err(ErrorData::invalid_params(
        format!(
            "Refusing to authorize an exception for '{}' over MCP: this transport cannot verify \
             human approval. Ask the user to run `slugrails exception add {} {} --reason=\"...\"` \
             in an interactive terminal.",
            req.file, req.file, req.guard_id
        ),
        None,
    ))
}

/// Lists active architectural exceptions for the project.
///
/// # Errors
/// Returns [`ErrorData`] if project path resolution or loading exceptions fails.
#[allow(clippy::needless_pass_by_value)]
pub fn handle_list_exceptions(
    req: ListExceptionsRequest,
) -> Result<Json<ListExceptionsResponse>, ErrorData> {
    let project_path = resolve_safe_root(req.project_path.as_deref())?;
    let exceptions =
        ProjectExceptions::load(&project_path).map_err(exception_store_err)?;
    let total = exceptions.exceptions.len();
    Ok(Json(ListExceptionsResponse {
        total,
        count: total,
        exceptions: exceptions.exceptions,
    }))
}

/// Revokes an authorized exception token.
///
/// # Errors
/// Returns [`ErrorData`] if project path resolution or revocation fails.
#[allow(clippy::needless_pass_by_value)]
pub fn handle_revoke_exception(
    req: RevokeExceptionRequest,
) -> Result<Json<RevokeExceptionResponse>, ErrorData> {
    if req.token.trim().is_empty() {
        return Err(ErrorData::invalid_params("Token cannot be empty", None));
    }
    let project_path = resolve_safe_root(req.project_path.as_deref())?;
    let mut exceptions =
        ProjectExceptions::load(&project_path).map_err(exception_store_err)?;
    let revoked = exceptions
        .revoke(&req.token)
        .map_err(|e| ErrorData::internal_error(format!("Revocation failed: {e}"), None))?;
    let msg = if revoked {
        "Exception token revoked successfully"
    } else {
        "Exception token not found"
    };
    Ok(Json(RevokeExceptionResponse {
        success: revoked,
        revoked,
        token: req.token,
        message: msg.to_string(),
    }))
}
