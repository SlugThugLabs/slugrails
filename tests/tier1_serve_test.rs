//! Tier 1: Feature coverage tests for `slugrails serve`.

mod common;

use common::{TestWorkspace, run_slugrails};

#[test]
fn test_serve_help_flag_displays_usage() {
    let ws = TestWorkspace::new("serve_help");
    let res = run_slugrails(&["serve", "--help"], &ws.root);
    assert_eq!(res.exit_code, 0);
    let output = format!("{}{}", res.stdout, res.stderr);
    assert!(output.contains("serve") || output.contains("MCP") || output.contains("--port"));
}

#[test]
fn test_serve_short_help_flag() {
    let ws = TestWorkspace::new("serve_short_help");
    let res = run_slugrails(&["serve", "-h"], &ws.root);
    assert_eq!(res.exit_code, 0);
}

#[test]
fn test_serve_stdio_handles_immediate_eof() {
    let ws = TestWorkspace::new("serve_eof");
    // Feed empty stdin (immediate EOF)
    let res = common::run_slugrails_with_stdin(&["serve"], &ws.root, "");
    // Server must cleanly terminate upon stdin EOF
    assert!(res.exit_code == 0 || res.exit_code == 1);
}

#[test]
fn test_serve_rejects_invalid_port_argument() {
    let ws = TestWorkspace::new("serve_bad_port");
    let res = run_slugrails(&["serve", "--port", "invalid_not_a_number"], &ws.root);
    // Invalid port must reject with exit code 2 or nonzero
    assert!(res.exit_code != 0);
}

#[test]
fn test_serve_rejects_unrecognized_flag() {
    let ws = TestWorkspace::new("serve_unknown_flag");
    let res = run_slugrails(&["serve", "--unknown-daemon-flag"], &ws.root);
    assert!(res.exit_code != 0);
}
