//! Integration tests for Git pre-commit hook installation and uninstallation.

#![allow(warnings, clippy::all, clippy::pedantic)]

use slugrails::cli::{install_hook, uninstall_hook};
use std::fs;
use std::process::ExitCode;

#[test]
fn test_hook_install_and_uninstall_lifecycle() {
    let temp_dir = std::env::temp_dir().join(format!("slugrails_hook_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    // 1. Fails when not a git repo
    let status = install_hook(&temp_dir);
    assert_eq!(status, ExitCode::from(1));

    // 2. Succeeds when .git exists
    let git_dir = temp_dir.join(".git");
    fs::create_dir_all(&git_dir).unwrap();

    let status = install_hook(&temp_dir);
    assert_eq!(status, ExitCode::SUCCESS);

    let hook_file = git_dir.join("hooks").join("pre-commit");
    assert!(hook_file.exists());
    let content = fs::read_to_string(&hook_file).unwrap();
    assert!(content.contains("slugrails check"));

    // 3. Idempotent install does not fail
    let status_idempotent = install_hook(&temp_dir);
    assert_eq!(status_idempotent, ExitCode::SUCCESS);

    // 4. Uninstall cleanly removes hook
    let status_uninstall = uninstall_hook(&temp_dir);
    assert_eq!(status_uninstall, ExitCode::SUCCESS);
    assert!(!hook_file.exists());

    // 5. Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}
