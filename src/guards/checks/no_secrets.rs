//! Hygiene guard: detects potential hardcoded secrets, API keys, and credentials.

use super::exception::has_valid_exception;
use crate::analyzer::metrics::{LiteralKind, extract_literals};
use crate::contract::ArchitectureContract;
use crate::guards::context::FileContext;
use crate::guards::traits::GuardCheck;
use crate::types::{Severity, Violation};
use regex::Regex;
use std::sync::LazyLock;

static RE_SECRETS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    let patterns = [
        r"-----BEGIN [A-Z0-9_-]+ PRIVATE KEY-----",
        r"\bgh[pousr]_[A-Za-z0-9_]{36,}\b",
        r"\bAKIA[0-9A-Z]{16}\b",
        r"\b(?:sk|rk)_live_[0-9a-zA-Z]{24,}\b",
        r"\bxox[baprs]-[0-9a-zA-Z]{10,}\b",
        r"(?i)(?:api_key|apikey|secret_key|auth_token)\s*[:=]\s*['\x22][A-Za-z0-9_\-]{16,}['\x22]",
    ];
    patterns.iter().filter_map(|p| Regex::new(p).ok()).collect()
});

pub struct NoSecretsCheck;

impl GuardCheck for NoSecretsCheck {
    fn id(&self) -> &'static str {
        "hygiene/no-secrets"
    }

    fn check(&self, ctx: &FileContext<'_>, _contract: &ArchitectureContract) -> Vec<Violation> {
        let mut violations = Vec::new();
        check_no_secrets(ctx, &mut violations);
        violations
    }
}

/// Scans source for hardcoded secrets, private keys, and known API token formats.
pub fn check_no_secrets(ctx: &FileContext<'_>, violations: &mut Vec<Violation>) {
    for lit in extract_literals(ctx.content) {
        if lit.kind != LiteralKind::String || ctx.is_test_line(lit.line) {
            continue;
        }

        let is_secret = RE_SECRETS
            .iter()
            .any(|re| re.is_match(&lit.value) || re.is_match(&lit.context));
        if is_secret && !has_valid_exception(ctx.file, &ctx.guard.id, ctx.content, ctx.exceptions) {
            let char_count = lit.value.chars().count();
            let masked = if char_count > 8 {
                let prefix: String = lit.value.chars().take(8).collect();
                format!("{prefix}...")
            } else {
                "***".to_string()
            };
            violations.push(Violation {
                guard_id: ctx.guard.id.clone(),
                file: ctx.rel_file.to_path_buf(),
                line: Some(lit.line),
                message: format!("Potential hardcoded secret or token detected: '{masked}'"),
                severity: Severity::Error,
                fix_suggestion: Some(
                    "Move secret to an environment variable or secure vault/keychain.".to_string(),
                ),
                rule_reference: Some(format!(
                    ".planning/ARCHITECTURE.md [enforce: {}]",
                    ctx.rule_name
                )),
            });
        }
    }
}
