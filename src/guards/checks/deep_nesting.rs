//! Complexity guard: flags functions exceeding maximum nesting depth.

use super::exception::has_valid_exception;
use crate::analyzer::metrics::nesting_depth;
use crate::contract::ArchitectureContract;
use crate::guards::context::FileContext;
use crate::guards::traits::GuardCheck;
use crate::types::{Severity, Violation};

pub struct DeepNestingCheck;

impl GuardCheck for DeepNestingCheck {
    fn id(&self) -> &'static str {
        "complexity/deep-nesting"
    }

    fn check(&self, ctx: &FileContext<'_>, contract: &ArchitectureContract) -> Vec<Violation> {
        let mut violations = Vec::new();
        check_deep_nesting(ctx, contract, &mut violations);
        violations
    }
}

/// Checks whether any function exceeds maximum allowed nesting depth (default 4).
pub fn check_deep_nesting(
    ctx: &FileContext<'_>,
    contract: &ArchitectureContract,
    violations: &mut Vec<Violation>,
) {
    let max_depth = contract
        .guard_settings
        .get("deep_nesting")
        .or_else(|| contract.guard_settings.get("complexity/deep-nesting"))
        .or_else(|| contract.guard_settings.get(&ctx.guard.id))
        .and_then(|v| v.get("max_depth"))
        .or_else(|| ctx.guard.default_params.get("max_depth"))
        .and_then(serde_json::Value::as_u64)
        .and_then(|u| usize::try_from(u).ok())
        .unwrap_or(4);

    for site in nesting_depth(ctx.content) {
        let is_rust = ctx.rel_file.extension().is_some_and(|ext| ext == "rs");
        let depth = if is_rust {
            site.max_depth.saturating_sub(1)
        } else {
            site.max_depth
        };
        if depth > max_depth
            && !ctx.is_test_line(site.line)
            && !has_valid_exception(ctx.file, &ctx.guard.id, ctx.content, ctx.exceptions)
        {
            violations.push(Violation {
                guard_id: ctx.guard.id.clone(),
                file: ctx.rel_file.to_path_buf(),
                line: Some(site.line),
                message: format!(
                    "Function '{}' exceeds maximum nesting depth ({depth}/{max_depth})",
                    site.name
                ),
                severity: Severity::Error,
                fix_suggestion: Some(
                    "Flatten control flow using guard clauses, early returns, or helper functions."
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
