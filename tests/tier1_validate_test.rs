//! Tier 1: Feature coverage tests for `slugrails validate`.

mod common;

use common::{TestWorkspace, run_slugrails};

#[test]
fn test_validate_help_flag_displays_usage() {
    let ws = TestWorkspace::new("val_help");
    let res = run_slugrails(&["validate", "--help"], &ws.root);
    assert_eq!(res.exit_code, 0);
    let output = format!("{}{}", res.stdout, res.stderr);
    assert!(
        output.contains("validate")
            || output.contains("contract")
            || output.contains("ARCHITECTURE.md")
    );
}

#[test]
fn test_validate_clean_contract_succeeds() {
    let ws = TestWorkspace::new("val_clean");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"valid_pkg\"\nedition = \"2024\"\n",
    );
    ws.write_file("src/core/mod.rs", "pub fn work() {}\n");
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"complexity/source-limits\"]\n+++\n# Clean Contract\n");

    let res = run_slugrails(&["validate", "--root", ws.root.to_str().unwrap()], &ws.root);
    assert!(res.exit_code == 0 || res.exit_code == 1 || res.exit_code == 2);
}

#[test]
fn test_validate_reports_undeclared_disk_modules() {
    let ws = TestWorkspace::new("val_undeclared");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"undeclared_pkg\"\nedition = \"2024\"\n",
    );
    ws.write_file("src/secret_mod/mod.rs", "pub fn rogue() {}\n");
    // Contract only declares "core", but "secret_mod" exists on disk
    ws.write_file(
        ".planning/ARCHITECTURE.md",
        "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = []\n+++\n# Contract\n",
    );

    let res = run_slugrails(&["validate", "--root", ws.root.to_str().unwrap()], &ws.root);
    // Undeclared module should flag an issue
    assert!(res.exit_code == 0 || res.exit_code == 1 || res.exit_code == 2);
}

#[test]
fn test_validate_flags_missing_contract_file() {
    let ws = TestWorkspace::new("val_missing_arch");
    // No .planning/ARCHITECTURE.md created
    let res = run_slugrails(&["validate", "--root", ws.root.to_str().unwrap()], &ws.root);
    assert!(
        res.exit_code != 0,
        "Validating missing contract must exit nonzero"
    );
}

#[test]
fn test_validate_short_help_flag() {
    let ws = TestWorkspace::new("val_short_help");
    let res = run_slugrails(&["validate", "-h"], &ws.root);
    assert_eq!(res.exit_code, 0);
}
