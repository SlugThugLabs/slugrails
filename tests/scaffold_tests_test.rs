use slugrails::cli::scaffold_tests::{
    TEST_HARNESS_TEMPLATE, run_scaffold_tests, scaffold_tests_harness,
};
use std::fs;
use std::process::ExitCode;
use tempfile::tempdir;

#[test]
fn test_scaffold_tests_harness_creates_slugrails_rs() {
    let dir = tempdir().expect("failed to create temp dir");
    let root = dir.path();

    let target_path = scaffold_tests_harness(root).expect("scaffold_tests_harness should succeed");

    assert_eq!(target_path, root.join("tests").join("slugrails.rs"));
    assert!(target_path.exists(), "tests/slugrails.rs must exist on disk");

    let content = fs::read_to_string(&target_path).expect("failed to read slugrails.rs");
    assert_eq!(
        content, TEST_HARNESS_TEMPLATE,
        "content must match TEST_HARNESS_TEMPLATE"
    );

    assert!(
        content.contains("fn test_architecture_contract_enforced()"),
        "must declare architectural test function"
    );
    assert!(
        content.contains("slugrails"),
        "must invoke slugrails"
    );
    assert!(
        content.contains(".planning"),
        "must reference .planning directory"
    );
    assert!(
        content.contains("ARCHITECTURE.md"),
        "must reference ARCHITECTURE.md"
    );
}

#[test]
fn test_scaffold_tests_harness_with_existing_tests_dir() {
    let dir = tempdir().expect("failed to create temp dir");
    let root = dir.path();
    let tests_dir = root.join("tests");
    fs::create_dir_all(&tests_dir).expect("failed to pre-create tests dir");

    // Put an existing test file in tests/
    let other_test = tests_dir.join("existing_test.rs");
    fs::write(&other_test, "// existing test\n").expect("failed to write existing test");

    let target_path = scaffold_tests_harness(root).expect("scaffold_tests_harness should succeed");
    assert!(target_path.exists());
    assert!(other_test.exists(), "existing tests must not be deleted");
}

#[test]
fn test_scaffold_tests_harness_idempotency() {
    let dir = tempdir().expect("failed to create temp dir");
    let root = dir.path();

    let first = scaffold_tests_harness(root).expect("first call");
    assert!(first.exists());

    let second = scaffold_tests_harness(root).expect("second call");
    assert_eq!(first, second);
    assert!(second.exists());

    let content = fs::read_to_string(&second).expect("read content");
    assert_eq!(content, TEST_HARNESS_TEMPLATE);
}

#[test]
fn test_run_scaffold_tests_cli_entrypoint() {
    let dir = tempdir().expect("failed to create temp dir");
    let root = dir.path();

    let args = vec![
        "slugrails".to_string(),
        "scaffold-tests".to_string(),
        "--project".to_string(),
        root.display().to_string(),
    ];

    let exit_code = run_scaffold_tests(args.into_iter());
    assert_eq!(exit_code, ExitCode::SUCCESS);
    assert!(root.join("tests").join("slugrails.rs").exists());
}
