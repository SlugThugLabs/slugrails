//! Tier 2: Boundary & Corner Cases for empty source trees and non-interactive exception attempts.

mod common;

use common::{TestWorkspace, run_slugrails};

#[test]
fn test_empty_source_tree_passes_cleanly() {
    let ws = TestWorkspace::new("empty_tree");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"empty_tree\"\nedition = \"2024\"\n",
    );
    ws.write_file(
        ".planning/ARCHITECTURE.md",
        "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = []\n+++\n# Contract\n",
    );
    // Ensure src/ exists but contains zero .rs files
    std::fs::create_dir_all(ws.root.join("src")).unwrap();

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert_eq!(res.exit_code, 0, "Empty source tree should pass cleanly");
    let out = format!("{}{}", res.stdout, res.stderr);
    assert!(
        out.contains("No source files")
            || out.contains("0 files")
            || out.contains("PASS")
            || res.is_success()
    );
}

#[test]
fn test_empty_rs_file_passes() {
    let ws = TestWorkspace::new("empty_file");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"empty_file\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"complexity/source-limits\"]\n+++\n# Contract\n");
    // Zero-byte .rs file
    ws.write_file("src/lib.rs", "");

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res.exit_code == 0 || res.exit_code == 2);
}

#[test]
fn test_whitespace_only_rs_file_passes() {
    let ws = TestWorkspace::new("whitespace_file");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"ws_file\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"complexity/source-limits\"]\n+++\n# Contract\n");
    ws.write_file("src/lib.rs", "   \n\t\n  \n");

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res.exit_code == 0 || res.exit_code == 2);
}

#[test]
fn test_noninteractive_exception_attempt_without_tty_aborts() {
    let ws = TestWorkspace::new("noninteractive_ex");
    // Autonomous script executing exception add without terminal TTY
    let res = run_slugrails(
        &[
            "exception",
            "add",
            "src/core/mod.rs",
            "complexity/source-limits",
            "--reason",
            "Autonomous agent trying to grant exception",
        ],
        &ws.root,
    );

    assert_eq!(
        res.exit_code, 2,
        "Autonomous agent must be denied granting exceptions (exit code 2)"
    );
    let err = format!("{}{}", res.stdout, res.stderr);
    assert!(err.contains("terminal") || err.contains("interactive") || err.contains("Autonomous"));
}

#[test]
fn test_exception_add_missing_reason_flag_fails() {
    let ws = TestWorkspace::new("ex_missing_reason");
    let res = run_slugrails(
        &["exception", "add", "src/lib.rs", "languages/rust/no-unwrap"],
        &ws.root,
    );

    assert!(
        res.exit_code != 0,
        "Missing reason must cause exception add to fail"
    );
}

#[test]
fn test_exception_add_missing_guard_argument_fails() {
    let ws = TestWorkspace::new("ex_missing_guard");
    let res = run_slugrails(&["exception", "add", "src/lib.rs"], &ws.root);

    assert!(res.exit_code != 0, "Missing guard parameter must fail");
}
