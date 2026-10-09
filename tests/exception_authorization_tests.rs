#![allow(warnings, clippy::all, clippy::pedantic)]

//! Authorization-boundary tests (audit SR-S03 / MT-12).
//!
//! An autonomous agent must not be able to grant itself a guard exception — neither
//! by setting an MCP request field, nor by invoking the CLI from a non-interactive
//! process.

use std::process::{Command, Stdio};

use slugrails::server::handlers::handle_add_exception;
use slugrails::server::AddExceptionRequest;
use slugrails::storage::ProjectExceptions;

#[test]
fn mcp_cannot_grant_an_exception_from_a_request_field() {
    let dir = tempfile::tempdir().expect("tempdir");
    let req = AddExceptionRequest {
        project_path: dir.path().to_string_lossy().to_string(),
        file: "src/legacy.rs".to_string(),
        guard_id: "complexity/source-limits".to_string(),
        reason: "Legacy code undergoing phased refactor".to_string(),
        human_confirmed: true,
    };

    let result = handle_add_exception(req);
    assert!(
        result.is_err(),
        "MCP must refuse to authorize an exception from a request field alone"
    );
    assert!(
        !dir.path().join(".planning/slugrails/exceptions.json").exists(),
        "a refused MCP request must not write an exception store"
    );
}

#[test]
fn cli_refuses_to_authorize_without_an_interactive_terminal() {
    let dir = tempfile::tempdir().expect("tempdir");
    let output = Command::new(env!("CARGO_BIN_EXE_slugrails"))
        .args([
            "exception",
            "add",
            "src/legacy.rs",
            "complexity/source-limits",
            "--reason=Legacy code undergoing phased refactor",
        ])
        .current_dir(dir.path())
        .stdin(Stdio::null())
        .output()
        .expect("run slugrails exception add");

    assert!(
        !output.status.success(),
        "a non-interactive `exception add` must not succeed"
    );
    assert!(
        !dir.path().join(".planning/slugrails/exceptions.json").exists(),
        "a refused CLI request must not write an exception store"
    );
    let exceptions = ProjectExceptions::load(dir.path()).expect("store readable");
    assert!(
        exceptions.exceptions.is_empty(),
        "a refused CLI request must not record an exception"
    );
}
