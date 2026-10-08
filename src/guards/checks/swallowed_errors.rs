//! Hygiene guard: flags swallowed errors and discarded results.

use super::exception::has_valid_exception;
use crate::analyzer::metrics::find_swallowed_errors;
use crate::contract::ArchitectureContract;
use crate::guards::context::FileContext;
use crate::guards::traits::GuardCheck;
use crate::types::{Severity, Violation};

pub struct SwallowedErrorsCheck;

impl GuardCheck for SwallowedErrorsCheck {
    fn id(&self) -> &'static str {
        "hygiene/swallowed-errors"
    }

    fn check(&self, ctx: &FileContext<'_>, _contract: &ArchitectureContract) -> Vec<Violation> {
        let mut violations = Vec::new();
        check_swallowed_errors(ctx, &mut violations);
        violations
    }
}

/// Checks for discarded results or swallowed errors (e.g. `let _ =`, empty match/catch).
pub fn check_swallowed_errors(ctx: &FileContext<'_>, violations: &mut Vec<Violation>) {
    for site in find_swallowed_errors(ctx.content) {
        if ctx.is_test_line(site.line) {
            continue;
        }
        if !has_valid_exception(ctx.file, &ctx.guard.id, ctx.content, ctx.exceptions) {
            violations.push(Violation {
                guard_id: ctx.guard.id.clone(),
                file: ctx.rel_file.to_path_buf(),
                line: Some(site.line),
                message: format!("Swallowed error or discarded result: '{}'", site.text),
                severity: Severity::Error,
                fix_suggestion: Some(
                    "Explicitly handle the error, propagate with '?', or log the failure."
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
