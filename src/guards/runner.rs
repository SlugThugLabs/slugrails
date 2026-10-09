//! Parallel guard execution runner.
//!
//! Dynamically routes active contract rules through the modular guard checks.

use super::context::FileContext;
use super::registry::DEFAULT_REGISTRY;
use crate::analyzer::walker::MAX_FILE_SIZE_BYTES;
use crate::analyzer::{is_test_path, test_region_lines};
use crate::contract::ArchitectureContract;
use crate::error::Result;
use crate::library::catalog::GuardCatalog;
use crate::storage::ProjectExceptions;
use crate::types::{GuardReport, SkipReason, SkippedFile, Violation};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Currently implemented active guard checks in the runner registry.
pub const IMPLEMENTED_GUARDS: &[&str] = &[
    "complexity/source-limits",
    "complexity/function-limits",
    "complexity/deep-nesting",
    "quality/no-stubs",
    "hygiene/swallowed-errors",
    "hygiene/no-secrets",
    "hygiene/no-debug-prints",
    "languages/rust/no-unwrap",
    "languages/rust/no-panic",
    "languages/rust/unsafe-policy",
    "structural/layer-dependencies",
    "structural/manifest-dependencies",
];

/// Known catalog guards not yet implemented in the runner registry.
pub const UNIMPLEMENTED_GUARDS: &[&str] = &[
    "structural/docs-drift",
    "hygiene/no-duplicates",
    "languages/rust/tracing-instrument",
    "languages/rust/runtime-leak",
];

/// Whether `guard_id` has a handler wired into the dispatcher.
#[must_use]
pub fn is_implemented(guard_id: &str) -> bool {
    DEFAULT_REGISTRY.is_implemented(guard_id)
}

/// Runs all applicable guards across the target list of files in parallel.
///
/// # Errors
///
/// Returns [`crate::error::SlugRailsError`] if critical evaluation fails.
pub fn run_guard_checks(
    project_root: &Path,
    files: &[PathBuf],
    contract: &ArchitectureContract,
    catalog: &GuardCatalog,
    exceptions: &ProjectExceptions,
) -> Result<GuardReport> {
    let start = Instant::now();

    let results: Vec<(Vec<Violation>, Option<SkippedFile>)> = files
        .par_iter()
        .map(|file| evaluate_file_guards(project_root, file, contract, catalog, exceptions))
        .collect();

    let mut violations = Vec::new();
    let mut skipped = Vec::new();
    for (mut file_violations, file_skip) in results {
        violations.append(&mut file_violations);
        if let Some(entry) = file_skip {
            skipped.push(entry);
        }
    }

    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    let (passed_tests, unevaluated_tests) = classify_rules(contract, catalog);

    tracing::debug!(
        files_checked = files.len(),
        violations_found = violations.len(),
        duration_ms,
        "Completed architecture guard scan"
    );

    Ok(GuardReport {
        project_root: project_root.to_path_buf(),
        total_files_checked: files.len(),
        violations,
        passed_tests,
        unevaluated_tests,
        active_exceptions: exceptions.exceptions.clone(),
        duration_ms,
        skipped,
        truncated: false,
        unreadable_entries: 0,
    })
}

/// Splits the contract's enforced rules into evaluated vs skipped.
#[must_use]
fn classify_rules(
    contract: &ArchitectureContract,
    catalog: &GuardCatalog,
) -> (Vec<String>, Vec<String>) {
    let mut evaluated = Vec::new();
    let mut skipped = Vec::new();

    for rule in &contract.enforce {
        let Some(guard) = catalog.resolve(rule) else {
            continue;
        };
        let will_run = if guard.id == "structural/layer-dependencies" {
            !contract.allowed_dependencies.is_empty()
        } else {
            is_implemented(&guard.id)
        };
        if will_run {
            evaluated.push(rule.clone());
        } else {
            skipped.push(rule.clone());
        }
    }

    (evaluated, skipped)
}

/// Evaluates a single file against all active contract rules.
fn evaluate_file_guards(
    project_root: &Path,
    file: &Path,
    contract: &ArchitectureContract,
    catalog: &GuardCatalog,
    exceptions: &ProjectExceptions,
) -> (Vec<Violation>, Option<SkippedFile>) {
    let mut violations = Vec::new();

    // Guard against reading oversized files (> 2 MiB) or non-regular files (FIFOs, device files).
    let Ok(meta) = fs::metadata(file) else {
        return (violations, skip(file, SkipReason::Unreadable));
    };
    if !meta.is_file() {
        return (violations, skip(file, SkipReason::NotRegularFile));
    }
    if meta.len() > MAX_FILE_SIZE_BYTES {
        tracing::warn!("Skipping oversized file: {}", file.display());
        return (violations, skip(file, SkipReason::Oversized));
    }

    let Ok(content) = fs::read_to_string(file) else {
        return (violations, skip(file, SkipReason::Unreadable));
    };

    let rel_file = file.strip_prefix(project_root).unwrap_or(file);
    let rel_str = rel_file.to_string_lossy();
    let test_regions = test_region_lines(&content);

    for rule_name in &contract.enforce {
        let Some(guard) = catalog.resolve(rule_name) else {
            continue;
        };

        let Some(check) = DEFAULT_REGISTRY.get(&guard.id) else {
            continue;
        };

        let ctx = FileContext {
            file,
            rel_file,
            rel_str: &rel_str,
            content: &content,
            is_test: is_test_path(&rel_str),
            test_regions: &test_regions,
            rule_name,
            guard,
            exceptions,
        };

        violations.extend(check.check(&ctx, contract));
    }

    (violations, None)
}

/// Records a discovered file that could not be inspected.
fn skip(file: &Path, reason: SkipReason) -> Option<SkippedFile> {
    Some(SkippedFile {
        file: file.to_path_buf(),
        reason,
    })
}
