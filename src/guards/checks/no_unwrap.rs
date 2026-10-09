//! Rust language guard: flags bare `.unwrap()`/`.expect()` in non-test sources.

use super::exception::has_valid_exception;
use crate::analyzer::find_unwrap_expect_calls;
use crate::contract::ArchitectureContract;
use crate::guards::context::FileContext;
use crate::guards::traits::GuardCheck;
use crate::types::{Severity, Violation};

pub struct NoUnwrapCheck;

impl GuardCheck for NoUnwrapCheck {
    fn id(&self) -> &'static str {
        "languages/rust/no-unwrap"
    }

    fn check(&self, ctx: &FileContext<'_>, _contract: &ArchitectureContract) -> Vec<Violation> {
        let mut violations = Vec::new();
        check_no_unwrap(ctx, &mut violations);
        violations
    }
}

/// Checks for bare unwrap/expect invocations in production Rust code.
pub fn check_no_unwrap(ctx: &FileContext<'_>, violations: &mut Vec<Violation>) {
    if ctx.rel_file.extension().is_none_or(|ext| ext != "rs") {
        return;
    }

    for (line, msg) in find_unwrap_expect_calls(ctx.content) {
        if ctx.is_test_line(line) {
            continue;
        }
        if !has_valid_exception(ctx.file, &ctx.guard.id, ctx.content, ctx.exceptions) {
            violations.push(Violation {
                guard_id: ctx.guard.id.clone(),
                file: ctx.rel_file.to_path_buf(),
                line: Some(line),
                message: msg,
                severity: Severity::Error,
                fix_suggestion: Some(
                    "Use '?' error propagation with thiserror or return a Result<T, E>."
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
