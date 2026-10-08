//! Complexity guard: flags functions whose line count exceeds configured limits.

use super::exception::has_valid_exception;
use crate::analyzer::metrics::function_spans;
use crate::contract::ArchitectureContract;
use crate::guards::context::FileContext;
use crate::guards::traits::GuardCheck;
use crate::types::{Severity, Violation};

pub struct FunctionLimitsCheck;

impl GuardCheck for FunctionLimitsCheck {
    fn id(&self) -> &'static str {
        "complexity/function-limits"
    }

    fn check(&self, ctx: &FileContext<'_>, contract: &ArchitectureContract) -> Vec<Violation> {
        let mut violations = Vec::new();
        check_function_limits(ctx, contract, &mut violations);
        violations
    }
}

/// Checks whether any function/method exceeds the max line length.
pub fn check_function_limits(
    ctx: &FileContext<'_>,
    contract: &ArchitectureContract,
    violations: &mut Vec<Violation>,
) {
    let max_lines = contract
        .guard_settings
        .get("function_limits")
        .or_else(|| contract.guard_settings.get("complexity/function-limits"))
        .or_else(|| contract.guard_settings.get(&ctx.guard.id))
        .and_then(|v| v.get("max_lines"))
        .or_else(|| ctx.guard.default_params.get("max_lines"))
        .and_then(serde_json::Value::as_u64)
        .and_then(|u| usize::try_from(u).ok())
        .unwrap_or(50);

    for span in function_spans(ctx.content) {
        let length = span.end_line.saturating_sub(span.start_line) + 1;
        if length > max_lines && !ctx.is_test_line(span.start_line) {
            if !has_valid_exception(ctx.file, &ctx.guard.id, ctx.content, ctx.exceptions) {
                violations.push(Violation {
                    guard_id: ctx.guard.id.clone(),
                    file: ctx.rel_file.to_path_buf(),
                    line: Some(span.start_line),
                    message: format!(
                        "Function '{}' has {length} lines (exceeds limit of {max_lines})",
                        span.name
                    ),
                    severity: Severity::Error,
                    fix_suggestion: Some(
                        "Refactor function into smaller, single-responsibility helper functions."
                            .to_string(),
                    ),
                    rule_reference: Some(format!(
                        ".planning/ARCHITECTURE.md [enforce: {}]",
                        ctx.rule_name
                    )),
                });
            }
        }
    }
}
