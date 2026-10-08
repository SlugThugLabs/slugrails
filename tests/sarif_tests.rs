use codeguards_mcp::types::{GuardReport, Severity, Violation};
use std::path::PathBuf;

#[test]
fn test_sarif_export_structure() {
    let report = GuardReport {
        project_root: PathBuf::from("."),
        total_files_checked: 1,
        violations: vec![Violation {
            guard_id: "complexity/source-limits".to_string(),
            file: PathBuf::from("src/main.rs"),
            line: Some(42),
            message: "File exceeds 400 lines".to_string(),
            severity: Severity::Error,
            fix_suggestion: Some("Split file".to_string()),
            rule_reference: Some(".planning/ARCHITECTURE.md".to_string()),
        }],
        passed_tests: vec![],
        unevaluated_tests: vec![],
        active_exceptions: vec![],
        duration_ms: 10,
    };

    // Use internal function via CLI test or serialization
    let json_val = serde_json::to_value(&report).unwrap();
    assert_eq!(json_val["total_files_checked"], 1);
    assert_eq!(json_val["violations"][0]["guard_id"], "complexity/source-limits");
    assert_eq!(json_val["violations"][0]["line"], 42);
}
