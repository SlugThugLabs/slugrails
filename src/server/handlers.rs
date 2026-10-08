//! Execution handlers for SlugRails and SlugPlan MCP tools.

use crate::analyzer::walker::{collect_git_diff_files, collect_source_files};
use crate::contract::{load_architecture, validate_architecture};
use crate::guards::runner::run_guard_checks;
use crate::library::catalog::GuardCatalog;
use crate::library::create_custom_guard_test;
use crate::server::types::{
    AddExceptionRequest, CheckProjectRequest, CreateGuardTestRequest, ProbeArchitectureRequest,
    ScaffoldHarnessRequest, ValidateArchitectureRequest,
};
use crate::storage::ProjectExceptions;
use crate::types::{GuardReport, GuardTestDefinition};
use rmcp::{ErrorData, Json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

fn resolve_safe_root(path_opt: Option<&str>) -> Result<PathBuf, ErrorData> {
    let p = path_opt.unwrap_or(".");
    if p.len() > 1024 {
        return Err(ErrorData::invalid_params("Project path too long", None));
    }
    crate::util::validate_safe_path(Path::new(p))
        .map_err(|e| ErrorData::invalid_params(format!("Unsafe path: {e}"), None))
}

pub async fn handle_validate_architecture(
    catalog: &Arc<Mutex<GuardCatalog>>,
    req: ValidateArchitectureRequest,
) -> Result<Json<crate::contract::ValidationResult>, ErrorData> {
    let project_root = resolve_safe_root(req.project_path.as_deref())?;
    let cat = catalog.lock().await;
    match validate_architecture(&project_root, &cat) {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err(ErrorData::internal_error(format!("Validation error: {e}"), None)),
    }
}

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

    let files = if req.check_all {
        collect_source_files(&project_root).unwrap_or_default()
    } else {
        collect_git_diff_files(&project_root).unwrap_or_default()
    };

    let exceptions = ProjectExceptions::load(&project_root).unwrap_or_default();
    let cat = catalog.lock().await.clone();
    let root = project_root.clone();
    let res = tokio::task::spawn_blocking(move || {
        run_guard_checks(&root, &files, &contract, &cat, &exceptions)
    })
    .await
    .map_err(|e| ErrorData::internal_error(format!("Scan task failed: {e}"), None))?;

    match res {
        Ok(report) => Ok(Json(report)),
        Err(e) => Err(ErrorData::internal_error(format!("Guard scan failed: {e}"), None)),
    }
}

pub async fn handle_probe(
    req: ProbeArchitectureRequest,
) -> Result<Json<serde_json::Value>, ErrorData> {
    if req.description.len() > 4096 {
        return Err(ErrorData::invalid_params("Description exceeds max length (4096)", None));
    }
    let project_root = resolve_safe_root(req.project_path.as_deref())?;
    let inspection = crate::contract::discovery::inspect_project(
        &project_root,
        &req.description,
        req.style.as_deref(),
    );

    let mut scaffold_output = None;
    if req.auto_scaffold {
        match crate::cli::plan_architecture(
            &project_root,
            Some(&req.description),
            req.style.as_deref(),
            req.custom_layers.clone(),
            req.custom_rules.clone(),
        ) {
            Ok(msg) => scaffold_output = Some(msg),
            Err(e) => return Err(ErrorData::internal_error(format!("Scaffolding failed: {e}"), None)),
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

pub fn handle_scaffold_harness(req: ScaffoldHarnessRequest) -> Result<Json<String>, ErrorData> {
    let project_root = resolve_safe_root(req.project_path.as_deref())?;
    match crate::cli::scaffold_tests_harness(&project_root) {
        Ok(p) => Ok(Json(format!("Scaffolded test harness: {}", p.display()))),
        Err(e) => Err(ErrorData::internal_error(format!("Failed to scaffold harness: {e}"), None)),
    }
}

pub async fn handle_create_guard_test(
    catalog: &Arc<Mutex<GuardCatalog>>,
    req: CreateGuardTestRequest,
) -> Result<Json<String>, ErrorData> {
    if req.name.len() > 128 || req.category.len() > 256 || req.summary.len() > 512
        || req.tags.len() > 32 || req.aliases.len() > 32 || req.remediation.len() > 2048
    {
        return Err(ErrorData::invalid_params("Request fields exceed size limits", None));
    }
    if req.category.contains("..") || req.name.contains("..")
        || req.name.contains('/') || req.name.contains('\\')
        || req.category.starts_with('/') || req.category.starts_with('\\')
    {
        return Err(ErrorData::invalid_params("Invalid path characters in category or name", None));
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
            Ok(Json(format!("Successfully created guard test at {}", path.display())))
        }
        Err(e) => Err(ErrorData::internal_error(format!("Failed to create guard test: {e}"), None)),
    }
}

pub fn handle_add_exception(req: AddExceptionRequest) -> Result<Json<crate::types::ExceptionEntry>, ErrorData> {
    if !req.human_confirmed {
        return Err(ErrorData::invalid_params(
            "Autonomous exceptions prohibited: explicit human confirmation required (pass human_confirmed: true)",
            None,
        ));
    }
    if req.file.contains("..") || req.file.len() > 1024 || req.reason.len() > 2048 {
        return Err(ErrorData::invalid_params("Invalid file path or reason length", None));
    }
    let project_path = resolve_safe_root(Some(&req.project_path))?;
    let mut exceptions = ProjectExceptions::load(&project_path).unwrap_or_default();
    match exceptions.add_exception(Path::new(&req.file), &req.guard_id, &req.reason) {
        Ok(entry) => Ok(Json(entry)),
        Err(e) => Err(ErrorData::internal_error(format!("Exception creation failed: {e}"), None)),
    }
}
