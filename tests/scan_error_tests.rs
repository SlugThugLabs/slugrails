#![allow(warnings, clippy::all, clippy::pedantic)]

//! Regression tests for the "false-success" class (audit SR-S01 / MT-07, MT-10).
//!
//! A scan that could not collect its inputs, or that could not load the exception
//! store, must never be reported to an MCP client as a clean pass.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use slugrails::library::builtins::get_builtin_guard_tests;
use slugrails::library::catalog::GuardCatalog;
use slugrails::server::handlers::handle_check_project;
use slugrails::server::CheckProjectRequest;
use slugrails::types::GuardTestDefinition;
use tokio::sync::Mutex;

const CONTRACT_NO_RULES: &str = "+++\nmodules = []\nlayers = []\nenforce = []\n+++\n\n# test contract\n";
const CONTRACT_STUBS: &str =
    "+++\nmodules = []\nlayers = []\nenforce = [\"quality/no-stubs\"]\n+++\n\n# test contract\n";

fn catalog() -> Arc<Mutex<GuardCatalog>> {
    let defs: Vec<(GuardTestDefinition, PathBuf)> = get_builtin_guard_tests()
        .into_iter()
        .map(|d| (d, PathBuf::from("unused.guard.json")))
        .collect();
    Arc::new(Mutex::new(GuardCatalog::from_definitions(&defs)))
}

/// A temp project containing `.planning/ARCHITECTURE.md`.
fn project(contract: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let planning = dir.path().join(".planning");
    std::fs::create_dir_all(&planning).expect("create .planning");
    std::fs::write(planning.join("ARCHITECTURE.md"), contract).expect("write contract");
    dir
}

fn request(path: &Path, check_all: bool) -> CheckProjectRequest {
    CheckProjectRequest {
        project_path: Some(path.to_string_lossy().to_string()),
        check_all,
    }
}

#[test]
fn collect_source_files_errors_on_missing_root() {
    // The unit-level guarantee the handler now relies on: collection failure is an Err.
    let missing = Path::new("/definitely/not/a/real/project/root");
    assert!(
        slugrails::analyzer::collect_source_files(missing).is_err(),
        "collecting from a missing root must be an error, not an empty list"
    );
}

#[tokio::test]
async fn missing_project_directory_is_an_error_not_a_pass() {
    let missing = PathBuf::from("/definitely/not/a/real/project/root");
    let result = handle_check_project(&catalog(), request(&missing, true)).await;
    assert!(
        result.is_err(),
        "a missing project directory must not produce a scan report"
    );

    let diff = handle_check_project(&catalog(), request(&missing, false)).await;
    assert!(
        diff.is_err(),
        "git-diff mode against a missing project must also error"
    );
}

#[tokio::test]
async fn empty_project_is_a_legitimate_pass() {
    let dir = project(CONTRACT_NO_RULES);
    let report = handle_check_project(&catalog(), request(dir.path(), true))
        .await
        .expect("a project with a valid contract and no sources is scannable")
        .0;

    assert_eq!(report.total_files_checked, 0, "no source files were present");
    assert!(report.violations.is_empty());
    assert!(report.is_pass(), "an empty but complete scan is a pass");
}

#[tokio::test]
async fn successful_scan_reports_violations() {
    let dir = project(CONTRACT_STUBS);
    let src = dir.path().join("src");
    std::fs::create_dir_all(&src).expect("create src");
    std::fs::write(src.join("stub.rs"), "fn a() { todo!(); }\n").expect("write source");

    let report = handle_check_project(&catalog(), request(dir.path(), true))
        .await
        .expect("scannable project")
        .0;

    assert_eq!(report.total_files_checked, 1, "the stub file must be inspected");
    assert!(
        report.violations.iter().any(|v| v.guard_id == "quality/no-stubs"),
        "the stub violation must be reported: {:?}",
        report.violations
    );
    assert!(!report.is_pass(), "a violating scan is not a pass");
}

#[tokio::test]
async fn unavailable_exception_store_is_an_error() {
    let dir = project(CONTRACT_NO_RULES);
    // A directory where the exceptions file is expected makes the read fail.
    let store = dir.path().join(".planning").join("slugrails").join("exceptions.json");
    std::fs::create_dir_all(&store).expect("create decoy store dir");

    let result = handle_check_project(&catalog(), request(dir.path(), true)).await;
    assert!(
        result.is_err(),
        "an unreadable exception store must error rather than scan with unknown exemptions"
    );
}

#[test]
fn default_store_refuses_to_persist_without_a_project_path() {
    // MT-10: an empty project_path used to write .planning/slugrails into the process CWD.
    let mut store = slugrails::storage::ProjectExceptions::default();
    let err = store.save();
    assert!(
        err.is_err(),
        "saving a store with an empty project path must fail rather than write to the CWD"
    );
}
