//! Tier 1: Feature coverage tests for `slugrails scaffold-tests`.

mod common;

use common::{TestWorkspace, run_slugrails};

#[test]
fn test_scaffold_tests_creates_slugrails_rs() {
    let ws = TestWorkspace::new("scaffold_create");
    let res = run_slugrails(
        &["scaffold-tests", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert_eq!(res.exit_code, 0);

    let test_file = ws.root.join("tests/slugrails.rs");
    assert!(test_file.exists(), "tests/slugrails.rs must be created");

    let content = ws.read_file("tests/slugrails.rs");
    assert!(
        content.contains("test_architecture_contract_enforced") || content.contains("slugrails")
    );
}

#[test]
fn test_scaffold_tests_alias_harness() {
    let ws = TestWorkspace::new("scaffold_alias");
    let res = run_slugrails(&["harness", "--root", ws.root.to_str().unwrap()], &ws.root);
    assert_eq!(res.exit_code, 0);

    let test_file = ws.root.join("tests/slugrails.rs");
    assert!(
        test_file.exists(),
        "harness alias must create tests/slugrails.rs"
    );
}

#[test]
fn test_scaffold_tests_preserves_existing_test_files() {
    let ws = TestWorkspace::new("scaffold_preserve");
    ws.write_file("tests/my_unit_test.rs", "// Pre-existing test\n");

    let res = run_slugrails(
        &["scaffold-tests", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert_eq!(res.exit_code, 0);

    assert!(ws.file_exists("tests/slugrails.rs"));
    assert!(ws.file_exists("tests/my_unit_test.rs"));
    let content = ws.read_file("tests/my_unit_test.rs");
    assert!(content.contains("Pre-existing test"));
}

#[test]
fn test_scaffold_tests_idempotent() {
    let ws = TestWorkspace::new("scaffold_idempotent");
    let res1 = run_slugrails(
        &["scaffold-tests", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert_eq!(res1.exit_code, 0);

    let res2 = run_slugrails(
        &["scaffold-tests", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert_eq!(res2.exit_code, 0);
    assert!(ws.file_exists("tests/slugrails.rs"));
}

#[test]
fn test_scaffold_tests_help_flag() {
    let ws = TestWorkspace::new("scaffold_help");
    let res = run_slugrails(&["scaffold-tests", "--help"], &ws.root);
    assert_eq!(res.exit_code, 0);
    let output = format!("{}{}", res.stdout, res.stderr);
    assert!(output.contains("scaffold-tests") || output.contains("slugrails.rs"));
}
