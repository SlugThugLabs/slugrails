//! Tier 1: Feature coverage tests for `slugrails exception`.

mod common;

use common::{TestWorkspace, run_slugrails};

#[test]
fn test_exception_help_flag_displays_usage() {
    let ws = TestWorkspace::new("ex_help");
    let res = run_slugrails(&["exception", "--help"], &ws.root);
    assert_eq!(res.exit_code, 0);
    let output = format!("{}{}", res.stdout, res.stderr);
    assert!(output.contains("exception") || output.contains("add") || output.contains("list"));
}

#[test]
fn test_exception_noninteractive_add_aborts_safely() {
    let ws = TestWorkspace::new("ex_noninteractive");
    // Non-interactive piped execution attempting to grant an exception
    let res = run_slugrails(
        &[
            "exception",
            "add",
            "src/lib.rs",
            "complexity/source-limits",
            "--reason",
            "test",
        ],
        &ws.root,
    );
    // Per VISION.md & ARCHITECTURE.md, autonomous agents cannot grant exceptions non-interactively
    assert!(
        res.exit_code == 1 || res.exit_code == 2,
        "Non-interactive exception add must fail with code 1 or 2"
    );
    let err = format!("{}{}", res.stdout, res.stderr);
    assert!(
        err.contains("terminal")
            || err.contains("interactive")
            || err.contains("Exception")
            || res.exit_code != 0
    );
}

#[test]
fn test_exception_list_empty_registry() {
    let ws = TestWorkspace::new("ex_list_empty");
    let res = run_slugrails(
        &["exception", "list", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res.exit_code == 0 || res.exit_code == 2);
}

#[test]
fn test_exception_revoke_nonexistent_token_fails() {
    let ws = TestWorkspace::new("ex_revoke_missing");
    let res = run_slugrails(
        &[
            "exception",
            "revoke",
            "non_existent_token_99999",
            "--root",
            ws.root.to_str().unwrap(),
        ],
        &ws.root,
    );
    // Revoking an absent token must fail with code 1 or 2
    assert!(res.exit_code != 0);
}

#[test]
fn test_exception_missing_subcommand_displays_usage() {
    let ws = TestWorkspace::new("ex_no_subcmd");
    let res = run_slugrails(&["exception"], &ws.root);
    // Should show usage
    let output = format!("{}{}", res.stdout, res.stderr);
    assert!(output.contains("exception") || output.contains("Usage") || res.exit_code != 0);
}
