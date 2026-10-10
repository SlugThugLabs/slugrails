//! Tier 1: Feature coverage tests for `slugrails hook`.

mod common;

use common::{TestWorkspace, run_slugrails};

#[test]
fn test_hook_help_flag_displays_usage() {
    let ws = TestWorkspace::new("hook_help");
    let res = run_slugrails(&["hook", "--help"], &ws.root);
    assert_eq!(res.exit_code, 0);
    let output = format!("{}{}", res.stdout, res.stderr);
    assert!(output.contains("hook") || output.contains("pre-commit") || output.contains("install"));
}

#[test]
fn test_hook_install_creates_pre_commit_script() {
    let ws = TestWorkspace::new("hook_install");
    ws.init_git();

    let res = run_slugrails(
        &["hook", "install", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert_eq!(res.exit_code, 0);

    let hook_file = ws.root.join(".git/hooks/pre-commit");
    assert!(hook_file.exists(), ".git/hooks/pre-commit must be created");

    let content = ws.read_file(".git/hooks/pre-commit");
    assert!(content.contains("slugrails") || content.contains("slugrails check"));
}

#[test]
fn test_hook_install_fails_outside_git_repository() {
    let ws = TestWorkspace::new("hook_no_git");
    // Workspace without .git
    let res = run_slugrails(
        &["hook", "install", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert_eq!(
        res.exit_code, 1,
        "Must exit 1 when not inside a git repository"
    );
}

#[test]
fn test_hook_install_idempotent() {
    let ws = TestWorkspace::new("hook_idempotent");
    ws.init_git();

    let res1 = run_slugrails(
        &["hook", "install", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert_eq!(res1.exit_code, 0);

    let res2 = run_slugrails(
        &["hook", "install", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert_eq!(res2.exit_code, 0);

    let content = ws.read_file(".git/hooks/pre-commit");
    // Ensure only one slugrails gate block exists
    let count = content.matches(">>> slugrails pre-commit gate >>>").count();
    assert!(count <= 1, "Must not duplicate pre-commit gate block");
}

#[test]
fn test_hook_uninstall_removes_slugrails_block() {
    let ws = TestWorkspace::new("hook_uninstall");
    ws.init_git();

    let res_install = run_slugrails(
        &["hook", "install", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert_eq!(res_install.exit_code, 0);

    let res_uninstall = run_slugrails(
        &["hook", "uninstall", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert_eq!(res_uninstall.exit_code, 0);

    let hook_file = ws.root.join(".git/hooks/pre-commit");
    if hook_file.exists() {
        let content = ws.read_file(".git/hooks/pre-commit");
        assert!(!content.contains("slugrails check"));
    }
}
