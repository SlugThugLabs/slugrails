//! Integration test suite covering SlugRails MCP server tool handlers.
//!
//! Validates:
//! - `test_check_project_handler`: verifies `handle_check_project` runs against a sample project directory.
//! - `test_slugplan_probe_handler`: verifies `handle_probe` returns architectural recommendations and layers.
//! - `test_scaffold_test_harness_handler`: verifies `handle_scaffold_harness` creates `tests/slugrails.rs`.
//! - `test_add_exception_handler`: verifies `handle_add_exception` refuses (MCP cannot authorize).
//! - `test_baseline_tamper_rejection`: verifies that modifying `ARCHITECTURE.md` without updating `contract.baseline` triggers tamper error.

#![allow(warnings, clippy::all, clippy::pedantic)]

use slugrails::contract::{load_architecture, save_contract_baseline, verify_contract_baseline};
use slugrails::library::catalog::GuardCatalog;
use slugrails::server::handlers::{
    handle_add_exception, handle_check_project, handle_probe, handle_scaffold_harness,
};
use slugrails::server::{
    AddExceptionRequest, CheckProjectRequest, ProbeArchitectureRequest, ScaffoldHarnessRequest,
};
use slugrails::storage::ProjectExceptions;
use slugrails::storage::baseline::{TAMPER_ERROR_MSG, get_contract_baseline_path};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::tempdir;
use tokio::sync::Mutex;

const SAMPLE_ARCH_FRONTMATTER: &str = r#"+++
modules = ["core"]
layers = ["foundation"]
enforce = ["source_limits"]

[allowed_dependencies]
core = []
+++

# Test Architecture Contract
"#;

#[tokio::test]
async fn test_check_project_handler() {
    let dir = tempdir().expect("failed to create tempdir");
    let project_root = dir.path();

    // 1. Scaffold valid contract
    let planning_dir = project_root.join(".planning");
    fs::create_dir_all(&planning_dir).expect("failed to create .planning dir");
    fs::write(
        planning_dir.join("ARCHITECTURE.md"),
        SAMPLE_ARCH_FRONTMATTER,
    )
    .expect("failed to write ARCHITECTURE.md");

    // 2. Scaffold valid source files
    let src_dir = project_root.join("src");
    fs::create_dir_all(&src_dir).expect("failed to create src dir");
    fs::write(
        src_dir.join("core.rs"),
        "pub fn compute() -> i32 {\n    42\n}\n",
    )
    .expect("failed to write src/core.rs");

    // 3. Initialize catalog and server handler
    let catalog = Arc::new(Mutex::new(GuardCatalog::default()));
    let req = CheckProjectRequest {
        project_path: Some(project_root.to_string_lossy().to_string()),
        check_all: true,
    };

    let result = handle_check_project(&catalog, req).await;
    assert!(
        result.is_ok(),
        "handle_check_project failed: {:?}",
        result.err()
    );

    let report = result.expect("unreachable").0;
    assert_eq!(report.total_files_checked, 1);
    assert!(report.is_pass(), "clean sample project should pass checks");
}

#[tokio::test]
async fn test_slugplan_probe_handler() {
    let dir = tempdir().expect("failed to create tempdir");
    let project_root = dir.path();

    let src_dir = project_root.join("src");
    fs::create_dir_all(&src_dir).expect("failed to create src dir");
    fs::write(src_dir.join("lib.rs"), "pub mod auth;\npub mod storage;\n")
        .expect("failed to write src/lib.rs");

    // Probe without auto-scaffolding
    let req = ProbeArchitectureRequest {
        description: "Event-driven high-throughput distributed microservice".to_string(),
        language: None,
        project_path: Some(project_root.to_string_lossy().to_string()),
        style: Some("layered".to_string()),
        custom_layers: None,
        custom_rules: None,
        auto_scaffold: false,
    };

    let result = handle_probe(req).await;
    assert!(
        result.is_ok(),
        "handle_probe should succeed: {:?}",
        result.err()
    );

    let val = result.expect("unreachable").0;
    assert_eq!(val["status"], "ready_to_scaffold");
    assert_eq!(
        val["description"],
        "Event-driven high-throughput distributed microservice"
    );

    let recommended_layers = val["recommended_layers"]
        .as_array()
        .expect("recommended_layers should be an array");
    assert!(
        !recommended_layers.is_empty(),
        "recommended_layers must not be empty"
    );

    let recommended_rules = val["recommended_rules"]
        .as_array()
        .expect("recommended_rules should be an array");
    assert!(
        !recommended_rules.is_empty(),
        "recommended_rules must not be empty"
    );

    let prompts = val["interview_prompts"]
        .as_array()
        .expect("interview_prompts should be an array");
    assert!(!prompts.is_empty(), "interview_prompts must not be empty");

    // Auto-scaffold probe
    let req_scaffold = ProbeArchitectureRequest {
        description: "Event-driven high-throughput distributed microservice".to_string(),
        language: None,
        project_path: Some(project_root.to_string_lossy().to_string()),
        style: Some("layered".to_string()),
        custom_layers: Some(vec!["transport".to_string(), "engine".to_string()]),
        custom_rules: Some(vec!["source_limits".to_string()]),
        auto_scaffold: true,
    };

    let result_scaffold = handle_probe(req_scaffold).await;
    assert!(result_scaffold.is_ok(), "handle_probe auto_scaffold failed");
    let val_scaffold = result_scaffold.expect("unreachable").0;
    assert_eq!(val_scaffold["status"], "scaffolded");
    assert!(
        project_root
            .join(".planning")
            .join("ARCHITECTURE.md")
            .exists()
    );
}

#[test]
fn test_scaffold_test_harness_handler() {
    let dir = tempdir().expect("failed to create tempdir");
    let project_root = dir.path();

    let req = ScaffoldHarnessRequest {
        project_path: Some(project_root.to_string_lossy().to_string()),
    };

    let result = handle_scaffold_harness(req);
    assert!(
        result.is_ok(),
        "handle_scaffold_harness failed: {:?}",
        result.err()
    );

    let harness_file = project_root.join("tests").join("slugrails.rs");
    assert!(
        harness_file.exists(),
        "tests/slugrails.rs harness must be created on disk"
    );

    let content = fs::read_to_string(&harness_file).expect("failed to read created harness");
    assert!(
        content.contains("test_architecture_contract_enforced"),
        "harness must contain architecture test entry point"
    );
}

#[test]
fn test_add_exception_handler() {
    let dir = tempdir().expect("failed to create tempdir");
    let project_root = dir.path();

    // 1. Without human confirmation -> MUST BE REJECTED
    let req_unconfirmed = AddExceptionRequest {
        project_path: project_root.to_string_lossy().to_string(),
        file: "src/legacy.rs".to_string(),
        guard_id: "complexity/source-limits".to_string(),
        reason: "Legacy code undergoing phased refactor".to_string(),
        human_confirmed: false,
    };

    let err_result = handle_add_exception(req_unconfirmed);
    assert!(
        err_result.is_err(),
        "Autonomous exception without human confirmation must be rejected"
    );

    // 2. Even with `human_confirmed: true` the request must be refused: the field is
    //    caller-controlled and cannot stand in for verified human approval (SR-S03).
    let req_confirmed = AddExceptionRequest {
        project_path: project_root.to_string_lossy().to_string(),
        file: "src/legacy.rs".to_string(),
        guard_id: "complexity/source-limits".to_string(),
        reason: "Legacy code undergoing phased refactor".to_string(),
        human_confirmed: true,
    };

    let confirmed_result = handle_add_exception(req_confirmed);
    assert!(
        confirmed_result.is_err(),
        "MCP must not grant an exception on a request field alone"
    );

    // 3. A refused request must not persist anything.
    let store_file = project_root.join(".planning/slugrails/exceptions.json");
    assert!(
        !store_file.exists(),
        "a refused exception request must not create an exception store"
    );
    let exceptions = ProjectExceptions::load(project_root).expect("failed to load exceptions");
    assert!(
        exceptions.exceptions.is_empty(),
        "a refused exception request must not record an exception"
    );
}

#[tokio::test]
async fn test_baseline_tamper_rejection() {
    let dir = tempdir().expect("failed to create tempdir");
    let project_root = dir.path();

    let planning_dir = project_root.join(".planning");
    fs::create_dir_all(&planning_dir).expect("failed to create .planning dir");
    let arch_file = planning_dir.join("ARCHITECTURE.md");

    // 1. Save valid initial architecture and approved baseline
    let original_content = SAMPLE_ARCH_FRONTMATTER.to_string();
    fs::write(&arch_file, &original_content).expect("failed to write ARCHITECTURE.md");
    save_contract_baseline(project_root, &original_content)
        .expect("failed to save approved baseline");

    let baseline_path = get_contract_baseline_path(project_root);
    assert!(
        baseline_path.exists(),
        "contract.baseline must exist after saving baseline"
    );

    // Baseline matches originally
    assert!(
        verify_contract_baseline(project_root, &original_content).is_ok(),
        "approved baseline must verify against matching content"
    );
    assert!(
        load_architecture(project_root).is_ok(),
        "load_architecture must succeed when baseline matches"
    );

    // 2. Tamper with ARCHITECTURE.md without updating contract.baseline
    let tampered_content = format!(
        "{}\n// Injected unauthorized agent bypass\n",
        SAMPLE_ARCH_FRONTMATTER
    );
    fs::write(&arch_file, &tampered_content).expect("failed to overwrite ARCHITECTURE.md");

    // 3. Verify direct baseline verification rejects tampered content
    let verify_err = verify_contract_baseline(project_root, &tampered_content);
    assert!(
        verify_err.is_err(),
        "verify_contract_baseline must reject tampered content"
    );

    // 4. Verify load_architecture rejects tampered ARCHITECTURE.md
    let load_err = load_architecture(project_root);
    assert!(
        load_err.is_err(),
        "load_architecture must reject tampered ARCHITECTURE.md"
    );
    let err_str = format!("{:?}", load_err.err());
    assert!(
        err_str.contains("contract.baseline") || err_str.contains(TAMPER_ERROR_MSG),
        "load_architecture error must reference baseline mismatch: {err_str}"
    );

    // 5. Verify handle_check_project handler rejects tampered project
    let catalog = Arc::new(Mutex::new(GuardCatalog::default()));
    let check_req = CheckProjectRequest {
        project_path: Some(project_root.to_string_lossy().to_string()),
        check_all: true,
    };

    let handler_err = handle_check_project(&catalog, check_req).await;
    assert!(
        handler_err.is_err(),
        "handle_check_project must return error on tampered contract"
    );
    let handler_err_msg = match handler_err {
        Err(e) => e.message,
        Ok(_) => panic!("expected error"),
    };
    assert!(
        handler_err_msg.contains("contract.baseline")
            || handler_err_msg.contains("Missing or invalid contract"),
        "handler error message must describe baseline or contract invalidity: {handler_err_msg}"
    );
}
