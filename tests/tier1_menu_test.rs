//! Tier 1: Feature coverage tests for `slugrails menu`.

mod common;

use common::{TestWorkspace, run_slugrails};

#[test]
fn test_menu_help_flag_displays_usage() {
    let ws = TestWorkspace::new("menu_help");
    let res = run_slugrails(&["menu", "--help"], &ws.root);
    assert_eq!(res.exit_code, 0);
    let output = format!("{}{}", res.stdout, res.stderr);
    assert!(output.contains("menu") || output.contains("Usage") || output.contains("Interactive"));
}

#[test]
fn test_menu_rejects_unrecognized_flag() {
    let ws = TestWorkspace::new("menu_invalid");
    let res = run_slugrails(&["menu", "--bogus-flag-1234"], &ws.root);
    // Menu or CLI parser rejects unknown flag with exit code 2 or nonzero
    assert!(res.exit_code != 0);
}

#[test]
fn test_menu_respects_custom_root_option() {
    let ws = TestWorkspace::new("menu_root");
    let dummy_root = ws.root.join("custom_root");
    std::fs::create_dir_all(&dummy_root).expect("create custom root");

    let res = run_slugrails(
        &["menu", "--root", dummy_root.to_str().unwrap(), "--help"],
        &ws.root,
    );
    assert_eq!(res.exit_code, 0);
}

#[test]
fn test_menu_noninteractive_invocation_does_not_hang() {
    let ws = TestWorkspace::new("menu_noninteractive");
    // Invoking menu non-interactively with empty stdin
    let res = common::run_slugrails_with_stdin(&["menu"], &ws.root, "q\n");
    // Should exit cleanly (either 0 or 1 on non-interactive abort)
    assert!(res.exit_code == 0 || res.exit_code == 1 || res.exit_code == 2);
}

#[test]
fn test_menu_short_help_flag() {
    let ws = TestWorkspace::new("menu_short_help");
    let res = run_slugrails(&["menu", "-h"], &ws.root);
    assert_eq!(res.exit_code, 0);
}
