//! Scaffold, custom test creation, and exception management actions for `SlugRails` MCP.

use crate::library::catalog::GuardCatalog;
use crate::library::create_custom_guard_test;
use crate::server::handlers::resolve_safe_root;
use crate::server::types::{AddExceptionRequest, CreateGuardTestRequest, ScaffoldHarnessRequest};
use crate::storage::ProjectExceptions;
use crate::types::GuardTestDefinition;
use rmcp::{ErrorData, Json};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Handles integration test harness scaffolding.
///
/// # Errors
///
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
///
/// Returns [`ErrorData`] if inputs fail parameter validation or file writing fails.
pub async fn handle_create_guard_test(
    catalog: &Arc<Mutex<GuardCatalog>>,
    req: CreateGuardTestRequest,
) -> Result<Json<String>, ErrorData> {
    if req.name.len() > 128
        || req.category.len() > 256
        || req.summary.len() > 512
        || req.tags.len() > 32
        || req.aliases.len() > 32
        || req.remediation.len() > 2048
    {
        return Err(ErrorData::invalid_params(
            "Request fields exceed size limits",
            None,
        ));
    }
    if req.category.contains("..")
        || req.name.contains("..")
        || req.name.contains('/')
        || req.name.contains('\\')
        || req.category.starts_with('/')
        || req.category.starts_with('\\')
    {
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
/// Returns [`ErrorData`] if not explicitly human confirmed or path resolution fails.
#[allow(clippy::needless_pass_by_value)]
pub fn handle_add_exception(
    req: AddExceptionRequest,
) -> Result<Json<crate::types::ExceptionEntry>, ErrorData> {
    if !req.human_confirmed {
        return Err(ErrorData::invalid_params(
            "Autonomous exceptions prohibited: explicit human confirmation required (pass human_confirmed: true)",
            None,
        ));
    }
    if req.file.contains("..")
        || Path::new(&req.file).is_absolute()
        || req.file.len() > 1024
        || req.reason.len() > 2048
    {
        return Err(ErrorData::invalid_params(
            "Invalid file path (must be relative without '..') or excessive length",
            None,
        ));
    }
    let project_path = resolve_safe_root(Some(&req.project_path))?;
    let mut exceptions = ProjectExceptions::load(&project_path).unwrap_or_default();
    match exceptions.add_exception(Path::new(&req.file), &req.guard_id, &req.reason) {
        Ok(entry) => Ok(Json(entry)),
        Err(e) => Err(ErrorData::internal_error(
            format!("Exception creation failed: {e}"),
            None,
        )),
    }
}
