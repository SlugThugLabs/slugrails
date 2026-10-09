//! Execution handlers for `SlugRails` and `SlugPlan` MCP tools.

pub use super::actions::{
    handle_add_exception, handle_create_guard_test, handle_list_exceptions,
    handle_revoke_exception, handle_scaffold_harness,
};
use crate::analyzer::walker::{collect_git_diff_files, collect_source_files};
use crate::contract::{load_architecture, validate_architecture};
use crate::guards::runner::run_guard_checks;
use crate::library::catalog::GuardCatalog;
use crate::server::types::{
    CheckProjectRequest, ProbeArchitectureRequest, ValidateArchitectureRequest,
};
use crate::storage::ProjectExceptions;
use crate::types::GuardReport;
use rmcp::{ErrorData, Json};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) fn resolve_safe_root(path_opt: Option<&str>) -> Result<PathBuf, ErrorData> {
    let p = path_opt.filter(|s| !s.trim().is_empty()).unwrap_or(".");
    if p.len() > 1024 {
        return Err(ErrorData::invalid_params("Project path too long", None));
    }
    crate::util::resolve_root(Path::new(p))
        .map_err(|e| ErrorData::invalid_params(format!("Unsafe path: {e}"), None))
}

/// MCP error for a failed source-file collection (no files were inspected).
pub(crate) fn collection_err(e: crate::error::SlugRailsError) -> ErrorData {
    ErrorData::internal_error(
        format!("Source collection failed; no files were inspected: {e}"),
        None,
    )
}

/// MCP error for a failed exception-store load.
pub(crate) fn exception_store_err(e: crate::error::SlugRailsError) -> ErrorData {
    ErrorData::internal_error(format!("Failed to load the exception store: {e}"), None)
}

/// Validates `.planning/ARCHITECTURE.md` against disk files and guard catalog.
///
/// # Errors
///
/// Returns [`ErrorData`] if path resolution or contract validation fails.
pub async fn handle_validate_architecture(
    catalog: &Arc<Mutex<GuardCatalog>>,
    req: ValidateArchitectureRequest,
) -> Result<Json<crate::contract::ValidationResult>, ErrorData> {
    let project_root = resolve_safe_root(req.project_path.as_deref())?;
    let cat = catalog.lock().await;
    match validate_architecture(&project_root, &cat) {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err(ErrorData::internal_error(
            format!("Validation error: {e}"),
            None,
        )),
    }
}

/// Executes all active guardrail checks on the project files.
///
/// # Errors
///
/// Returns [`ErrorData`] if loading architecture, file collection, or guard scan fails.
pub async fn handle_check_project(
    catalog: &Arc<Mutex<GuardCatalog>>,
    req: CheckProjectRequest,
) -> Result<Json<GuardReport>, ErrorData> {
    let project_root = resolve_safe_root(req.project_path.as_deref())?;
    let contract = load_architecture(&project_root).map_err(|e| {
        ErrorData::invalid_params(
            format!("Missing or invalid contract: {e}. Run 'slugrails init' first."),
            None,
        )
    })?;

    let check_all = req.check_all;
    let root = project_root.clone();
    let collection = tokio::task::spawn_blocking(move || {
        if check_all {
            collect_source_files(&root)
        } else {
            collect_git_diff_files(&root)
        }
    })
    .await
    .map_err(|e| ErrorData::internal_error(format!("File collection task failed: {e}"), None))?
    .map_err(collection_err)?;
    let truncated = collection.truncated;
    let unreadable_entries = collection.unreadable_entries;
    let files = collection.files;

    let exceptions = ProjectExceptions::load(&project_root).map_err(exception_store_err)?;
    let cat = catalog.clone().lock_owned().await;
    let root = project_root.clone();
    let res = tokio::task::spawn_blocking(move || {
        run_guard_checks(&root, &files, &contract, &cat, &exceptions)
    })
    .await
    .map_err(|e| ErrorData::internal_error(format!("Scan task failed: {e}"), None))?;

    let mut report =
        res.map_err(|e| ErrorData::internal_error(format!("Guard scan failed: {e}"), None))?;
    report.truncated = truncated;
    report.unreadable_entries = unreadable_entries;
    Ok(Json(report))
}

/// Inspects project structure and scaffolds proposed architecture.
///
/// # Errors
///
/// Returns [`ErrorData`] if description exceeds length limits or scaffolding encounters an error.
#[allow(clippy::unused_async)]
pub async fn handle_probe(
    req: ProbeArchitectureRequest,
) -> Result<Json<serde_json::Value>, ErrorData> {
    if req.description.len() > 4096 {
        return Err(ErrorData::invalid_params(
            "Description exceeds max length (4096)",
            None,
        ));
    }
    let project_root = resolve_safe_root(req.project_path.as_deref())?;
    let inspection = crate::contract::discovery::inspect_project(
        &project_root,
        &req.description,
        req.style.as_deref(),
    );

    let mut scaffold_output = None;
    if req.auto_scaffold {
        match crate::contract::plan_architecture(
            &project_root,
            Some(&req.description),
            req.style.as_deref(),
            req.custom_layers.clone(),
            req.custom_rules.clone(),
        ) {
            Ok(msg) => scaffold_output = Some(msg),
            Err(e) => {
                return Err(ErrorData::internal_error(
                    format!("Scaffolding failed: {e}"),
                    None,
                ));
            }
        }
    }

    Ok(Json(serde_json::json!({
        "status": if scaffold_output.is_some() { "scaffolded" } else { "ready_to_scaffold" },
        "description": req.description,
        "detected_language": inspection.detected_language,
        "existing_modules": inspection.existing_modules,
        "recommended_layers": req.custom_layers.unwrap_or(inspection.candidate_layers),
        "recommended_rules": req.custom_rules.unwrap_or(inspection.recommended_rules),
        "interview_prompts": inspection.interview_prompts,
        "candidate_contract": inspection.candidate_contract,
        "scaffold_result": scaffold_output,
        "guidance": "Discuss boundaries and terms with user in chat. When agreed, call with auto_scaffold: true."
    })))
}
