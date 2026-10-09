#![allow(warnings, clippy::all, clippy::pedantic)]

//! Inspection-completeness regression tests (audit SR-S04 / MT-08).
//!
//! A scan must distinguish "inspected" from "skipped", and an incomplete scan must
//! never be reported as a verified pass.

use std::path::PathBuf;

use slugrails::contract::ArchitectureContract;
use slugrails::guards::run_guard_checks;
use slugrails::library::builtins::get_builtin_guard_tests;
use slugrails::library::catalog::GuardCatalog;
use slugrails::storage::ProjectExceptions;
use slugrails::types::{
    GuardReport, GuardTestDefinition, ScanOutcome, Severity, SkipReason, SkippedFile, Violation,
};

fn catalog() -> GuardCatalog {
    let defs: Vec<(GuardTestDefinition, PathBuf)> = get_builtin_guard_tests()
        .into_iter()
        .map(|d| (d, PathBuf::from("unused.guard.json")))
        .collect();
    GuardCatalog::from_definitions(&defs)
}

fn contract() -> ArchitectureContract {
    ArchitectureContract {
        enforce: vec!["quality/no-stubs".to_string()],
        ..Default::default()
    }
}

#[test]
fn oversized_files_are_reported_as_skipped_not_silently_dropped() {
    let dir = tempfile::tempdir().expect("tempdir");
    let big = dir.path().join("big.rs");
    let mut content = String::with_capacity(2 * 1024 * 1024 + 64);
    while content.len() < 2 * 1024 * 1024 + 64 {
        content.push_str("// padding padding padding padding padding\n");
    }
    content.push_str("fn a() { todo!(); }\n");
    std::fs::write(&big, content).expect("write big file");

    let report = run_guard_checks(
        dir.path(),
        &[big],
        &contract(),
        &catalog(),
        &ProjectExceptions::default(),
    )
    .expect("scan runs");

    assert_eq!(report.skipped.len(), 1, "the oversized file must be recorded");
    assert_eq!(report.skipped[0].reason, SkipReason::Oversized);
    assert_eq!(report.outcome(), ScanOutcome::Incomplete);
    assert!(
        !report.is_pass(),
        "an incomplete scan must never be reported as a pass"
    );
}

#[test]
fn non_regular_files_are_recorded() {
    let dir = tempfile::tempdir().expect("tempdir");
    let as_dir = dir.path().join("looks_like.rs");
    std::fs::create_dir_all(&as_dir).expect("create dir");

    let report = run_guard_checks(
        dir.path(),
        &[as_dir],
        &contract(),
        &catalog(),
        &ProjectExceptions::default(),
    )
    .expect("scan runs");

    assert_eq!(report.skipped.len(), 1);
    assert_eq!(report.skipped[0].reason, SkipReason::NotRegularFile);
    assert_eq!(report.outcome(), ScanOutcome::Incomplete);
}

#[test]
fn unreadable_files_are_recorded() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("does_not_exist.rs");

    let report = run_guard_checks(
        dir.path(),
        &[missing],
        &contract(),
        &catalog(),
        &ProjectExceptions::default(),
    )
    .expect("scan runs");

    assert_eq!(report.skipped.len(), 1);
    assert_eq!(report.skipped[0].reason, SkipReason::Unreadable);
    assert_eq!(report.outcome(), ScanOutcome::Incomplete);
}

#[test]
fn the_walker_reports_completeness_for_a_normal_tree() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("a.rs"), "fn a() {}\n").expect("write");
    std::fs::write(dir.path().join("notes.txt"), "ignored by extension\n").expect("write");

    let collected = slugrails::analyzer::collect_source_files(dir.path()).expect("collect");
    assert!(collected.files.iter().any(|p| p.ends_with("a.rs")));
    assert!(
        !collected.files.iter().any(|p| p.ends_with("notes.txt")),
        "extension exclusions are intentional, not inspection failures"
    );
    assert!(!collected.truncated);
    assert_eq!(collected.unreadable_entries, 0);
}

#[test]
fn scan_outcome_model_matches_the_specification() {
    // Complete + no error-severity violation.
    let clean = GuardReport::default();
    assert_eq!(clean.outcome(), ScanOutcome::Passed);
    assert!(clean.is_pass());

    // Warnings do not fail a complete scan.
    let mut warned = GuardReport::default();
    warned.violations.push(Violation {
        guard_id: "g".to_string(),
        file: PathBuf::from("f.rs"),
        line: None,
        message: "w".to_string(),
        severity: Severity::Warning,
        fix_suggestion: None,
        rule_reference: None,
    });
    assert_eq!(warned.outcome(), ScanOutcome::Passed);

    // Complete + error-severity violation.
    let mut violating = GuardReport::default();
    violating.violations.push(Violation {
        guard_id: "g".to_string(),
        file: PathBuf::from("f.rs"),
        line: None,
        message: "e".to_string(),
        severity: Severity::Error,
        fix_suggestion: None,
        rule_reference: None,
    });
    assert_eq!(violating.outcome(), ScanOutcome::Violations);
    assert!(!violating.is_pass());

    // Incomplete by truncation.
    let mut truncated = GuardReport::default();
    truncated.truncated = true;
    assert_eq!(truncated.outcome(), ScanOutcome::Incomplete);
    assert!(!truncated.is_pass());

    // Incomplete by traversal error count.
    let mut unreadable = GuardReport::default();
    unreadable.unreadable_entries = 1;
    assert_eq!(unreadable.outcome(), ScanOutcome::Incomplete);

    // Incomplete by per-file skip.
    let mut skipped = GuardReport::default();
    skipped.skipped.push(SkippedFile {
        file: PathBuf::from("x.rs"),
        reason: SkipReason::Unreadable,
    });
    assert_eq!(skipped.outcome(), ScanOutcome::Incomplete);
    assert!(!skipped.is_pass());
}
