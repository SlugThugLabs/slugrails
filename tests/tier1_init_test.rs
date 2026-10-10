//! Tier 1: Feature coverage tests for `slugrails init`.

mod common;

use common::{TestWorkspace, run_slugrails};

#[test]
fn test_init_creates_planning_dir_and_contract() {
    let ws = TestWorkspace::new("init_basic");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"sample\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    ws.write_file("src/auth/mod.rs", "pub fn login() {}\n");
    ws.write_file("src/billing/mod.rs", "pub fn charge() {}\n");

    let res = run_slugrails(&["init", "--root", ws.root.to_str().unwrap()], &ws.root);
    assert_eq!(res.exit_code, 0);

    let arch_path = ws.root.join(".planning").join("ARCHITECTURE.md");
    assert!(
        arch_path.exists(),
        ".planning/ARCHITECTURE.md must be created"
    );

    let content = std::fs::read_to_string(&arch_path).expect("read contract");
    assert!(content.contains("+++"));
    assert!(content.contains("\"auth\"") || content.contains("auth"));
    assert!(content.contains("\"billing\"") || content.contains("billing"));
}

#[test]
fn test_init_does_not_overwrite_existing_contract() {
    let ws = TestWorkspace::new("init_idempotent");
    ws.write_file(
        ".planning/ARCHITECTURE.md",
        "# Custom User Contract\nNever overwrite.\n",
    );

    let res = run_slugrails(&["init", "--root", ws.root.to_str().unwrap()], &ws.root);
    assert_eq!(res.exit_code, 0);

    let content = ws.read_file(".planning/ARCHITECTURE.md");
    assert!(content.contains("Custom User Contract"));
    assert!(content.contains("Never overwrite"));
}

#[test]
fn test_init_help_flag_displays_usage() {
    let ws = TestWorkspace::new("init_help");
    let res = run_slugrails(&["init", "--help"], &ws.root);
    assert_eq!(res.exit_code, 0);
    let output = format!("{}{}", res.stdout, res.stderr);
    assert!(output.contains("init") || output.contains("ARCHITECTURE.md"));
}

#[test]
fn test_init_scaffolds_default_guards_in_contract() {
    let ws = TestWorkspace::new("init_guards");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"guards_test\"\nedition = \"2024\"\n",
    );
    ws.write_file("src/lib.rs", "pub fn run() {}\n");

    let res = run_slugrails(&["init", "--root", ws.root.to_str().unwrap()], &ws.root);
    assert_eq!(res.exit_code, 0);

    let content = ws.read_file(".planning/ARCHITECTURE.md");
    assert!(content.contains("complexity/source-limits") || content.contains("enforce"));
}

#[test]
fn test_init_creates_slugrails_project_directory() {
    let ws = TestWorkspace::new("init_storage_dir");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"store_test\"\nedition = \"2024\"\n",
    );

    let res = run_slugrails(&["init", "--root", ws.root.to_str().unwrap()], &ws.root);
    assert_eq!(res.exit_code, 0);

    let plan_dir = ws.root.join(".planning");
    assert!(plan_dir.exists());
}
