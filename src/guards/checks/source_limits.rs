//! Complexity guard: flags files exceeding maximum code-line limits.

use super::exception::has_valid_exception;
use crate::analyzer::count_code_lines;
use crate::contract::ArchitectureContract;
use crate::guards::context::FileContext;
use crate::guards::traits::GuardCheck;
use crate::types::{Severity, Violation};

pub struct SourceLimitsCheck;

impl GuardCheck for SourceLimitsCheck {
    fn id(&self) -> &'static str {
        "complexity/source-limits"
    }

    fn check(&self, ctx: &FileContext<'_>, contract: &ArchitectureContract) -> Vec<Violation> {
        let mut violations = Vec::new();
        check_source_limits(ctx, contract, &mut violations);
        violations
    }
}

/// Checks whether a file's code lines exceed the configured threshold.
pub fn check_source_limits(
    ctx: &FileContext<'_>,
    contract: &ArchitectureContract,
    violations: &mut Vec<Violation>,
) {
    if ctx.is_test {
        return;
    }
    let max_lines = usize::try_from(
        contract
            .guard_settings
            .get("source_limits")
            .and_then(|v| v.get("max_lines"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(400),
    )
    .ok()
    .unwrap_or(usize::MAX);

    let code_lines = count_code_lines(ctx.content);
    if code_lines > max_lines
        && !has_valid_exception(ctx.file, &ctx.guard.id, ctx.content, ctx.exceptions)
    {
        violations.push(Violation {
            guard_id: ctx.guard.id.clone(),
            file: ctx.rel_file.to_path_buf(),
            line: Some(1),
            message: format!("File has {code_lines} code lines (exceeds limit of {max_lines})"),
            severity: Severity::Error,
            fix_suggestion: Some(ctx.guard.summary.clone()),
            rule_reference: Some(format!(
                ".planning/ARCHITECTURE.md [enforce: {}]",
                ctx.rule_name
            )),
        });
    }
}
