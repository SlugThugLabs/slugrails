//! Rust language guard: checks for unexempted `unsafe` blocks and functions.

use super::exception::has_valid_exception;
use crate::analyzer::tokenizer::tokenize_source;
use crate::contract::ArchitectureContract;
use crate::guards::context::FileContext;
use crate::guards::traits::GuardCheck;
use crate::types::{Severity, Violation};
use regex::Regex;
use std::sync::LazyLock;

static RE_UNSAFE: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r"\bunsafe\b").ok());

pub struct UnsafePolicyCheck;

impl GuardCheck for UnsafePolicyCheck {
    fn id(&self) -> &'static str {
        "languages/rust/unsafe-policy"
    }

    fn check(&self, ctx: &FileContext<'_>, _contract: &ArchitectureContract) -> Vec<Violation> {
        let mut violations = Vec::new();
        check_unsafe_policy(ctx, &mut violations);
        violations
    }
}

/// Checks for unexempted `unsafe` keyword usage in Rust production code.
pub fn check_unsafe_policy(ctx: &FileContext<'_>, violations: &mut Vec<Violation>) {
    if ctx.rel_file.extension().is_none_or(|ext| ext != "rs") {
        return;
    }

    let Some(re) = RE_UNSAFE.as_ref() else {
        return;
    };

    for stripped in tokenize_source(ctx.content) {
        if ctx.is_test_line(stripped.line_number) {
            continue;
        }

        if re.is_match(&stripped.code_only)
            && !has_valid_exception(ctx.file, &ctx.guard.id, ctx.content, ctx.exceptions)
        {
            violations.push(Violation {
                guard_id: ctx.guard.id.clone(),
                file: ctx.rel_file.to_path_buf(),
                line: Some(stripped.line_number),
                message: format!("Unsafe code detected: '{}'", stripped.raw.trim()),
                severity: Severity::Error,
                fix_suggestion: Some(
                    "Replace unsafe block with safe abstraction or record an approved architecture exception."
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
