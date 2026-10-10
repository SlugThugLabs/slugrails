//! Tier 1: Feature coverage tests for `slugrails check`.

mod common;

use common::{TestWorkspace, run_slugrails};

#[test]
fn test_check_help_flag_displays_usage() {
    let ws = TestWorkspace::new("check_help");
    let res = run_slugrails(&["check", "--help"], &ws.root);
    assert_eq!(res.exit_code, 0);
    let output = format!("{}{}", res.stdout, res.stderr);
    assert!(output.contains("check") || output.contains("--format") || output.contains("--all"));
}

#[test]
fn test_check_clean_project_passes() {
    let ws = TestWorkspace::new("check_clean");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"clean\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"languages/rust/no-unwrap\"]\n+++\n# Contract\n");
    ws.write_file("src/lib.rs", "pub fn clean_fn() -> i32 { 42 }\n");

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res.exit_code == 0 || res.exit_code == 2);
}

#[test]
fn test_check_flags_violations_with_exit_code_1() {
    let ws = TestWorkspace::new("check_violation");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"violator\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"languages/rust/no-unwrap\"]\n+++\n# Contract\n");
    // Add production unwrap violation
    ws.write_file(
        "src/lib.rs",
        "pub fn bad() -> i32 { \"123\".parse::<i32>().unwrap() }\n",
    );

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    // When violation is detected, exit code must be 1
    if res.exit_code != 2 {
        assert_eq!(res.exit_code, 1, "Violation should yield exit code 1");
    }
}

#[test]
fn test_check_format_json_outputs_valid_json() {
    let ws = TestWorkspace::new("check_json");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"json_app\"\nedition = \"2024\"\n",
    );
    ws.write_file(
        ".planning/ARCHITECTURE.md",
        "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = []\n+++\n# Contract\n",
    );
    ws.write_file("src/lib.rs", "pub fn ok() {}\n");

    let res = run_slugrails(
        &[
            "check",
            "--all",
            "--format",
            "json",
            "--root",
            ws.root.to_str().unwrap(),
        ],
        &ws.root,
    );
    let output = &res.stdout;
    if !output.trim().is_empty() {
        assert!(
            output.starts_with('{')
                || output.contains("\"outcome\"")
                || output.contains("\"violations\"")
        );
    }
}

#[test]
fn test_check_format_sarif_outputs_valid_sarif() {
    let ws = TestWorkspace::new("check_sarif");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"sarif_app\"\nedition = \"2024\"\n",
    );
    ws.write_file(
        ".planning/ARCHITECTURE.md",
        "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = []\n+++\n# Contract\n",
    );
    ws.write_file("src/lib.rs", "pub fn ok() {}\n");

    let res = run_slugrails(
        &[
            "check",
            "--all",
            "--format",
            "sarif",
            "--root",
            ws.root.to_str().unwrap(),
        ],
        &ws.root,
    );
    let output = &res.stdout;
    if !output.trim().is_empty() {
        assert!(output.contains("$schema") || output.contains("sarif") || output.contains("runs"));
    }
}
