//! Core domain models and reporting types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Severity level of a guard violation.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational notice; does not cause verification failure.
    Info,
    /// Warning diagnostic; reported but does not fail by default.
    Warning,
    /// Critical error; causes verification failure.
    Error,
}

/// A specific violation detected in a file or project structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Violation {
    /// Unique identifier of the violated guard.
    pub guard_id: String,
    /// Relative or absolute path of the offending file.
    pub file: PathBuf,
    /// Line number where violation occurred, if applicable.
    pub line: Option<usize>,
    /// Human-readable explanation of the violation.
    pub message: String,
    /// Severity classification of the violation.
    pub severity: Severity,
    /// Actionable remediation advice for resolving the violation.
    pub fix_suggestion: Option<String>,
    /// Reference section in ARCHITECTURE.md or guidelines.
    pub rule_reference: Option<String>,
}

/// A verified user-authorized exception entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExceptionEntry {
    /// Cryptographic exception token.
    pub token: String,
    /// File path covered by the exception.
    pub file: PathBuf,
    /// Guard identifier covered by the exception.
    pub guard_id: String,
    /// Human justification for the exception.
    pub reason: String,
    /// Timestamp when the exception was granted.
    pub granted_at: String,
}

/// Aggregated report from running guard tests.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct GuardReport {
    /// Root path of the analyzed project.
    pub project_root: PathBuf,
    /// Total count of source files inspected.
    pub total_files_checked: usize,
    /// All detected violations across inspected files.
    pub violations: Vec<Violation>,
    /// Rules a handler actually evaluated in this run.
    pub passed_tests: Vec<String>,
    /// Enabled rules that no handler evaluated.
    pub unevaluated_tests: Vec<String>,
    /// Currently active and validated exceptions.
    pub active_exceptions: Vec<ExceptionEntry>,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
}

impl GuardReport {
    /// Determines whether the scan passed with zero error-level violations.
    #[must_use]
    pub fn is_pass(&self) -> bool {
        !self
            .violations
            .iter()
            .any(|violation| violation.severity == Severity::Error)
    }

    /// Computes the total number of error-level violations.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|violation| violation.severity == Severity::Error)
            .count()
    }

    /// Computes the total number of warning-level violations.
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|violation| violation.severity == Severity::Warning)
            .count()
    }
}

/// Reusable Guard Test definition schema (.guard.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardTestDefinition {
    /// Unique guard test identifier.
    pub id: String,
    /// Descriptive name of the guard test.
    pub name: String,
    /// Category classification (e.g., hygiene, structural).
    pub category: String,
    /// Schema or guard version string.
    pub version: String,
    /// High-level summary of what the guard verifies.
    pub summary: String,
    /// Classification tags for discovery and filtering.
    pub tags: Vec<String>,
    /// Alternate identifiers or aliases for the guard.
    pub aliases: Vec<String>,
    /// Engine used to evaluate the guard (e.g., builtin, custom).
    pub engine: String,
    /// Default parameters supplied to the guard engine.
    #[serde(default)]
    pub default_params: BTreeMap<String, serde_json::Value>,
    /// Remediation instructions displayed upon violation.
    pub remediation: String,
}

/// Outcome of an architectural guardrail scan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanOutcome {
    /// All guards passed cleanly.
    Passed,
    /// One or more violations were detected.
    Violations(Vec<Violation>),
    /// Required checks could not be evaluated.
    Incomplete(Vec<String>),
    /// Guard execution failed with an error message.
    Failed(String),
}

/// Source coordinate span identifying a region in a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// 1-based start line coordinate.
    pub start_line: usize,
    /// 1-based start column coordinate.
    pub start_col: usize,
    /// 1-based end line coordinate.
    pub end_line: usize,
    /// 1-based end column coordinate.
    pub end_col: usize,
}
