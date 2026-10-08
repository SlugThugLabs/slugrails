//! Hygiene guard: flags leftover debug print statements in non-test sources.

use super::exception::has_valid_exception;
use crate::analyzer::find_debug_prints;
use crate::guards::context::FileContext;
use crate::types::{Severity, Violation};

/// Checks for debug print statements (`dbg!`, `println!`, `console.log`) in production code.
pub fn check_debug_prints(ctx: &FileContext<'_>, violations: &mut Vec<Violation>) {
    for (line, msg) in find_debug_prints(ctx.content) {
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
                    "Replace debug print with structured tracing::info/debug or remove before committing."
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
