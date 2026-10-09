//! Rust language guard: flags panic!() macro calls in production code.

use super::exception::has_valid_exception;
use crate::contract::ArchitectureContract;
use crate::guards::context::FileContext;
use crate::guards::traits::GuardCheck;
use crate::types::{Severity, Violation};

pub struct NoPanicCheck;

impl GuardCheck for NoPanicCheck {
    fn id(&self) -> &'static str {
        "languages/rust/no-panic"
    }

    fn check(&self, ctx: &FileContext<'_>, _contract: &ArchitectureContract) -> Vec<Violation> {
        let mut violations = Vec::new();
        check_no_panic(ctx, &mut violations);
        violations
    }
}

pub fn check_no_panic(ctx: &FileContext<'_>, violations: &mut Vec<Violation>) {
    if ctx.rel_file.extension().is_none_or(|ext| ext != "rs") {
        return;
    }

    for (line_idx, line) in ctx.content.lines().enumerate() {
        let line_num = line_idx + 1;
        if ctx.is_test_line(line_num) {
            continue;
        }

        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }

        if (trimmed.contains("panic!(") || trimmed.contains("panic! ("))
            && !has_valid_exception(ctx.file, &ctx.guard.id, ctx.content, ctx.exceptions)
        {
            violations.push(Violation {
                guard_id: ctx.guard.id.clone(),
                file: ctx.rel_file.to_path_buf(),
                line: Some(line_num),
                message: "Production code must not invoke panic!()".to_string(),
                severity: Severity::Error,
                fix_suggestion: Some(
                    "Return a structured Result<T, E> error or propagate with '?'.".to_string(),
                ),
                rule_reference: Some(format!(
                    ".planning/ARCHITECTURE.md [enforce: {}]",
                    ctx.rule_name
                )),
            });
        }
    }
}
