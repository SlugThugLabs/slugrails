//! Quality guard: flags placeholder implementations and stub macros (`todo!`, `unimplemented!`, `panic!("not implemented")`).

use super::exception::has_valid_exception;
use crate::analyzer::find_stub_markers;
use crate::contract::ArchitectureContract;
use crate::guards::context::FileContext;
use crate::guards::traits::GuardCheck;
use crate::types::{Severity, Violation};

pub struct NoStubsCheck;

impl GuardCheck for NoStubsCheck {
    fn id(&self) -> &'static str {
        "quality/no-stubs"
    }

    fn check(&self, ctx: &FileContext<'_>, _contract: &ArchitectureContract) -> Vec<Violation> {
        let mut violations = Vec::new();
        check_no_stubs(ctx, &mut violations);
        violations
    }
}

/// Checks for stub macros and placeholder implementations in production code.
pub fn check_no_stubs(ctx: &FileContext<'_>, violations: &mut Vec<Violation>) {
    for site in find_stub_markers(ctx.content) {
        if ctx.is_test_line(site.line) {
            continue;
        }
        if !has_valid_exception(ctx.file, &ctx.guard.id, ctx.content, ctx.exceptions) {
            violations.push(Violation {
                guard_id: ctx.guard.id.clone(),
                file: ctx.rel_file.to_path_buf(),
                line: Some(site.line),
                message: format!("Unimplemented stub detected: '{}'", site.text),
                severity: Severity::Error,
                fix_suggestion: Some(
                    "Replace stub placeholder with complete production implementation before committing."
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
