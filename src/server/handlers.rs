//! Execution handlers for SlugRails and SlugPlan MCP tools.

use crate::analyzer::walker::{collect_modified_files, collect_source_files};
use crate::contract::{load_architecture, validate_architecture};
use crate::guards::runner::run_guard_checks;
use crate::library::catalog::GuardCatalog;
use crate::library::create_custom_guard_test;
use crate::server::types::{
    AddExceptionRequest, CheckProjectRequest, CreateGuardTestRequest, ProbeArchitectureRequest,
    ValidateArchitectureRequest,
};
use crate::storage::ProjectExceptions;
use crate::types::{GuardReport, GuardTestDefinition};
use rmcp::{ErrorData, Json};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle_validate_architecture(
    catalog: &Arc<Mutex<GuardCatalog>>,
    req: ValidateArchitectureRequest,
) -> Result<Json<crate::contract::ValidationResult>, ErrorData> {
    let path_str = req.project_path.as_deref().unwrap_or(".");
    if path_str.len() > 1024 {
        return Err(ErrorData::invalid_params("Project path too long", None));
    }
    let project_root = match crate::util::validate_safe_path(Path::new(path_str)) {
        Ok(p) => p,
        Err(e) => return Err(ErrorData::invalid_params(format!("Unsafe path: {e}"), None)),
    };
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
    let path_str = req.project_path.as_deref().unwrap_or(".");
    if path_str.len() > 1024 {
        return Err(ErrorData::invalid_params("Project path too long", None));
    }
    let project_root = match crate::util::validate_safe_path(Path::new(path_str)) {
        Ok(p) => p,
        Err(e) => return Err(ErrorData::invalid_params(format!("Unsafe path: {e}"), None)),
    };

    let contract = match load_architecture(&project_root) {
        Ok(c) => c,
        Err(e) => {
            return Err(ErrorData::invalid_params(
                format!("Missing or invalid contract: {e}. Run 'slugrails init' first."),
                None,
            ));
        }
    };

    let files = if req.check_all {
        collect_source_files(&project_root)
    } else {
        collect_modified_files(&project_root)
    };

    let exceptions = ProjectExceptions::load(&project_root).unwrap_or_default();
    let cat = catalog.lock().await;

    match run_guard_checks(&project_root, &files, &contract, &cat, &exceptions) {
        Ok(report) => Ok(Json(report)),
        Err(e) => Err(ErrorData::internal_error(format!("Guard scan failed: {e}"), None)),
    }
}

pub async fn handle_probe(
    req: ProbeArchitectureRequest,
) -> Result<Json<serde_json::Value>, ErrorData> {
    let lang = req.language.as_deref().unwrap_or("rust").to_lowercase();
    let mut recommended_rules = vec![
        "complexity/source-limits",
        "quality/no-stubs",
        "hygiene/no-secrets",
        "hygiene/no-debug-prints",
    ];
    if lang.contains("rust") {
        recommended_rules.push("languages/rust/no-unwrap");
    }

    let result = serde_json::json!({
        "status": "ready_to_scaffold",
        "description": req.description,
        "recommended_layers": ["domain", "application", "infrastructure"],
        "recommended_rules": recommended_rules,
        "scaffold_command": "slugrails init",
        "planning_directory": ".planning/slugplan/",
        "rails_directory": ".planning/slugrails/",
        "guidance": "Run 'slugrails init' to establish the baseline contract. Ensure source files stay <= 200 LOC."
    });

    Ok(Json(result))
}

pub async fn handle_create_guard_test(
    catalog: &Arc<Mutex<GuardCatalog>>,
    req: CreateGuardTestRequest,
) -> Result<Json<String>, ErrorData> {
    if req.name.len() > 128 || req.category.len() > 256 || req.summary.len() > 512 {
        return Err(ErrorData::invalid_params("Request fields too large", None));
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
    let project_path = match crate::util::validate_safe_path(Path::new(&req.project_path)) {
        Ok(p) => p,
        Err(e) => return Err(ErrorData::invalid_params(format!("Unsafe project path: {e}"), None)),
    };
    let mut exceptions = ProjectExceptions::load(&project_path).unwrap_or_default();
    match exceptions.add_exception(Path::new(&req.file), &req.guard_id, &req.reason) {
        Ok(entry) => Ok(Json(entry)),
        Err(e) => Err(ErrorData::internal_error(format!("Exception creation failed: {e}"), None)),
    }
}
