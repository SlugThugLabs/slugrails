use slugrails::cli::fix::auto_remediate_violations;
use slugrails::types::{Severity, Violation};
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_auto_remediate_debug_prints() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("dirty.rs");

    let original = "pub fn add(a: i32, b: i32) -> i32 {\n    dbg!(a);\n    a + b\n}\n";
    fs::write(&file, original).unwrap();

    let violations = vec![Violation {
        guard_id: "hygiene/no-debug-prints".to_string(),
        file: PathBuf::from("dirty.rs"),
        line: Some(2),
        message: "debug statement found".to_string(),
        severity: Severity::Error,
        fix_suggestion: Some("Remove debug print".to_string()),
        rule_reference: None,
    }];

    let fixed = auto_remediate_violations(dir.path(), &violations);
    assert_eq!(fixed, 1);

    let updated = fs::read_to_string(&file).unwrap();
    assert!(!updated.contains("dbg!(a);"));
    assert!(updated.contains("pub fn add(a: i32, b: i32) -> i32 {"));
    assert!(updated.contains("a + b"));
}
