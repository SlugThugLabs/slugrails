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
    Info,
    Warning,
    Error,
}

/// A specific violation detected in a file or project structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Violation {
    pub guard_id: String,
    pub file: PathBuf,
    pub line: Option<usize>,
    pub message: String,
    pub severity: Severity,
    pub fix_suggestion: Option<String>,
    pub rule_reference: Option<String>,
}

/// A verified user-authorized exception entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExceptionEntry {
    pub token: String,
    pub file: PathBuf,
    pub guard_id: String,
    pub reason: String,
    pub granted_at: String,
}

/// Why a discovered source file was not inspected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SkipReason {
    /// Larger than the per-file inspection limit.
    Oversized,
    /// Not a regular file (FIFO, socket, device, or directory).
    NotRegularFile,
    /// Could not be read (permissions, encoding, or I/O error).
    Unreadable,
}

/// A discovered source file that could not be inspected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SkippedFile {
    pub file: PathBuf,
    pub reason: SkipReason,
}

/// Overall completeness and severity of a scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScanOutcome {
    /// Every collected file was inspected and no error-severity violation was found.
    Passed,
    /// Every collected file was inspected and error-severity violations were found.
    Violations,
    /// The scan finished but some inputs were skipped or truncated: the result is partial.
    Incomplete,
    /// The scan could not run. Surfaced as a transport error, never as a report.
    Failed,
}

/// Candidate source files collected for a scan, together with completeness data.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ScanCollection {
    pub files: Vec<PathBuf>,
    /// True when the file budget was reached and the walk stopped early.
    #[serde(default)]
    pub truncated: bool,
    /// Traversal entries that could not be read at all.
    #[serde(default)]
    pub unreadable_entries: usize,
}

/// Aggregated report from running guard tests.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct GuardReport {
    pub project_root: PathBuf,
    pub total_files_checked: usize,
    pub violations: Vec<Violation>,
    /// Rules a handler actually evaluated in this run.
    pub passed_tests: Vec<String>,
    /// Enabled rules that no handler evaluated, so this run says nothing about
    /// them. Kept separate from `passed_tests` so a guard with no implementation
    /// cannot be reported as a success.
    pub unevaluated_tests: Vec<String>,
    pub active_exceptions: Vec<ExceptionEntry>,
    pub duration_ms: u64,
    /// Files that were discovered but not inspected, with the reason.
    #[serde(default)]
    pub skipped: Vec<SkippedFile>,
    /// True when the file budget was reached and the walk stopped early.
    #[serde(default)]
    pub truncated: bool,
    /// Traversal entries (directories or entries) that could not be read.
    #[serde(default)]
    pub unreadable_entries: usize,
}

impl GuardReport {
    /// Whether the scan was complete and found no error-severity violation.
    ///
    /// An incomplete scan is never a pass, however few violations it found.
    #[must_use]
    pub fn is_pass(&self) -> bool {
        self.outcome() == ScanOutcome::Passed
    }

    /// Completeness/severity model for this scan.
    ///
    /// `ScanOutcome::Failed` is not produced by a report: a scan that cannot run
    /// returns a transport error instead of a report (see the audit's SR-S01 fix).
    #[must_use]
    pub fn outcome(&self) -> ScanOutcome {
        if self.truncated || self.unreadable_entries > 0 || !self.skipped.is_empty() {
            ScanOutcome::Incomplete
        } else if self
            .violations
            .iter()
            .any(|v| v.severity == Severity::Error)
        {
            ScanOutcome::Violations
        } else {
            ScanOutcome::Passed
        }
    }

    #[must_use]
    pub fn error_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == Severity::Error)
            .count()
    }

    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == Severity::Warning)
            .count()
    }
}

/// Reusable Guard Test definition schema (.guard.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardTestDefinition {
    pub id: String,
    pub name: String,
    pub category: String,
    pub version: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub aliases: Vec<String>,
    pub engine: String,
    #[serde(default)]
    pub default_params: BTreeMap<String, serde_json::Value>,
    pub remediation: String,
}
