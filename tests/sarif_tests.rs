#![allow(warnings, clippy::all, clippy::pedantic)]

use slugrails::cli::sarif::format_sarif;
use slugrails::types::{GuardReport, Severity, Violation};
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
        skipped: vec![],
        truncated: false,
        unreadable_entries: 0,
    };

    // Verify report serialization retains backwards compatibility
    let json_val = serde_json::to_value(&report).unwrap();
    assert_eq!(json_val["total_files_checked"], 1);
    assert_eq!(
        json_val["violations"][0]["guard_id"],
        "complexity/source-limits"
    );
    assert_eq!(json_val["violations"][0]["line"], 42);

    // Call real format_sarif and assert SARIF 2.1.0 schema properties
    let sarif = format_sarif(&report);

    // SARIF 2.1.0 root properties
    assert_eq!(sarif["version"], "2.1.0");
    assert_eq!(
        sarif["$schema"],
        "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json"
    );

    // Runs array
    let runs = sarif["runs"]
        .as_array()
        .expect("runs should be a JSON array");
    assert_eq!(runs.len(), 1);

    // Tool driver
    let driver = &runs[0]["tool"]["driver"];
    assert_eq!(driver["name"], "SlugRails");
    assert_eq!(driver["semanticVersion"], env!("CARGO_PKG_VERSION"));
    assert_eq!(
        driver["informationUri"],
        "https://github.com/SlugThugLabs/SlugRails"
    );

    // Results array
    let results = runs[0]["results"]
        .as_array()
        .expect("results should be a JSON array");
    assert_eq!(results.len(), 1);

    let res = &results[0];
    assert_eq!(res["ruleId"], "complexity/source-limits");
    assert_eq!(res["level"], "error");
    assert_eq!(res["message"]["text"], "File exceeds 400 lines");

    // Locations and physical location
    let locations = res["locations"]
        .as_array()
        .expect("locations should be an array");
    assert_eq!(locations.len(), 1);
    assert_eq!(
        locations[0]["physicalLocation"]["artifactLocation"]["uri"],
        "src/main.rs"
    );
    assert_eq!(locations[0]["physicalLocation"]["region"]["startLine"], 42);

    // Fix suggestions
    let fixes = res["fixes"]
        .as_array()
        .expect("fixes should be an array when fix_suggestion is present");
    assert_eq!(fixes.len(), 1);
    assert_eq!(fixes[0]["description"]["text"], "Split file");
}

#[test]
fn test_sarif_empty_and_multiple_severities() {
    // Empty report produces valid SARIF 2.1.0 with empty results
    let empty_report = GuardReport {
        project_root: PathBuf::from("."),
        total_files_checked: 0,
        violations: vec![],
        passed_tests: vec![],
        unevaluated_tests: vec![],
        active_exceptions: vec![],
        duration_ms: 0,
        skipped: vec![],
        truncated: false,
        unreadable_entries: 0,
    };
    let empty_sarif = format_sarif(&empty_report);
    assert_eq!(empty_sarif["version"], "2.1.0");
    assert_eq!(
        empty_sarif["$schema"],
        "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json"
    );
    let empty_results = empty_sarif["runs"][0]["results"].as_array().unwrap();
    assert!(empty_results.is_empty());

    // Multiple violations with Warning and Info severities and without fixes
    let multi_report = GuardReport {
        project_root: PathBuf::from("."),
        total_files_checked: 2,
        violations: vec![
            Violation {
                guard_id: "hygiene/no-debug-prints".to_string(),
                file: PathBuf::from("src/util.rs"),
                line: None,
                message: "dbg! statement found".to_string(),
                severity: Severity::Warning,
                fix_suggestion: None,
                rule_reference: None,
            },
            Violation {
                guard_id: "quality/no-stubs".to_string(),
                file: PathBuf::from("src/stub.rs"),
                line: Some(15),
                message: "todo! found".to_string(),
                severity: Severity::Info,
                fix_suggestion: None,
                rule_reference: None,
            },
        ],
        passed_tests: vec![],
        unevaluated_tests: vec![],
        active_exceptions: vec![],
        duration_ms: 5,
        skipped: vec![],
        truncated: false,
        unreadable_entries: 0,
    };

    let multi_sarif = format_sarif(&multi_report);
    let results = multi_sarif["runs"][0]["results"].as_array().unwrap();
    assert_eq!(results.len(), 2);

    // Warning maps to "warning", None line defaults to 1
    assert_eq!(results[0]["ruleId"], "hygiene/no-debug-prints");
    assert_eq!(results[0]["level"], "warning");
    assert_eq!(
        results[0]["locations"][0]["physicalLocation"]["region"]["startLine"],
        1
    );
    assert!(results[0].get("fixes").is_none());

    // Info maps to "note"
    assert_eq!(results[1]["ruleId"], "quality/no-stubs");
    assert_eq!(results[1]["level"], "note");
    assert_eq!(
        results[1]["locations"][0]["physicalLocation"]["region"]["startLine"],
        15
    );
    assert!(results[1].get("fixes").is_none());
}
