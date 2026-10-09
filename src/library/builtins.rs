//! Built-in guard test definitions embedded directly into the binary.

use super::builtin_rules::{GuardSpec, complexity_guards, structural_guards};
use crate::types::GuardTestDefinition;

fn hygiene_guards() -> Vec<GuardTestDefinition> {
    vec![
        GuardSpec {
            id: "quality/no-stubs",
            name: "no_stubs",
            category: "quality",
            summary: "Rejects unimplemented stub macros (todo!, unimplemented!) in production",
            tags: &["stubs", "todo", "quality"],
            aliases: &["no_stubs", "check_no_stubs", "forbid_stubs"],
            engine: "no_stubs",
            params: &[],
            remediation: "Provide complete production implementation or remove unused stub.",
        }.into_definition(),
        GuardSpec {
            id: "hygiene/swallowed-errors",
            name: "swallowed_errors",
            category: "hygiene",
            summary: "Flags swallowed errors, discarded results, or empty error catch/match arms",
            tags: &["errors", "swallowed", "discarded", "exceptions"],
            aliases: &["swallowed_errors", "empty_catch", "discarded_results"],
            engine: "swallowed_errors",
            params: &[],
            remediation: "Explicitly handle errors, propagate with '?', or log failures.",
        }.into_definition(),
        GuardSpec {
            id: "hygiene/no-secrets",
            name: "no_secrets",
            category: "hygiene",
            summary: "Detects hardcoded API keys, private tokens, and credentials",
            tags: &["security", "secrets", "tokens", "keys"],
            aliases: &["no_secrets", "no_hardcoded_secrets", "secret_scanner"],
            engine: "no_secrets",
            params: &[],
            remediation: "Extract hardcoded secret into environment variable or configuration file.",
        }.into_definition(),
        GuardSpec {
            id: "hygiene/no-duplicates",
            name: "no_duplicates",
            category: "hygiene",
            summary: "Disallows duplicate test function names and duplicated git commit subjects",
            tags: &["dedupe", "tests", "git"],
            aliases: &["no_duplicates", "check_no_duplicates"],
            engine: "no_duplicates",
            params: &[],
            remediation: "Rename duplicate test function to describe specific unique behavior.",
        }.into_definition(),
        GuardSpec {
            id: "hygiene/no-debug-prints",
            name: "no_debug_prints",
            category: "hygiene",
            summary: "Rejects leftover debugging statements in production",
            tags: &["logging", "debug", "prints"],
            aliases: &["no_debug_prints", "debug_statements"],
            engine: "no_debug_prints",
            params: &[],
            remediation: "Replace debug print statement with structured tracing or remove it before committing.",
        }.into_definition(),
    ]
}

fn rust_guards() -> Vec<GuardTestDefinition> {
    vec![
        GuardSpec {
            id: "languages/rust/no-unwrap",
            name: "no_unwrap",
            category: "languages/rust",
            summary: "Prohibits bare .unwrap() and .expect() calls in production Rust source code",
            tags: &["rust", "errors", "unwrap", "safety"],
            aliases: &["no_unwrap", "forbid_unwrap"],
            engine: "rust_no_unwrap",
            params: &[],
            remediation: "Use ? error propagation with thiserror or return a structured Result<T, E>.",
        }.into_definition(),
        GuardSpec {
            id: "languages/rust/tracing-instrument",
            name: "tracing_instrument",
            category: "languages/rust",
            summary: "Requires #[tracing::instrument] on public API functions",
            tags: &["rust", "tracing", "observability"],
            aliases: &["tracing_instrument", "require_tracing"],
            engine: "rust_tracing_instrument",
            params: &[],
            remediation: "Add #[tracing::instrument] above public entrypoint function.",
        }.into_definition(),
        GuardSpec {
            id: "languages/rust/unsafe-policy",
            name: "unsafe_policy",
            category: "languages/rust",
            summary: "Enforces #![forbid(unsafe_code)] or strict bounds on unsafe blocks",
            tags: &["rust", "unsafe", "security"],
            aliases: &["unsafe_policy", "forbid_unsafe"],
            engine: "rust_unsafe_policy",
            params: &[],
            remediation: "Remove unsafe block or add crate-level #![forbid(unsafe_code)] directive.",
        }.into_definition(),
        GuardSpec {
            id: "languages/rust/no-panic",
            name: "no_panic",
            category: "languages/rust",
            summary: "Prohibits panic!() macro invocations in production Rust source code",
            tags: &["rust", "panic", "safety"],
            aliases: &["no_panic", "forbid_panic"],
            engine: "rust_no_panic",
            params: &[],
            remediation: "Replace panic!() with structured Result error propagation.",
        }.into_definition(),
        GuardSpec {
            id: "languages/rust/runtime-leak",
            name: "runtime_leak",
            category: "languages/rust",
            summary: "Prevents async runtime crates (tokio, wasmtime, rayon) from leaking into pure domain/contracts",
            tags: &["rust", "tokio", "leak", "architecture"],
            aliases: &["runtime_leak", "tokio_leak_guard"],
            engine: "rust_runtime_leak",
            params: &[],
            remediation: "Isolate async runtime calls into the transport or infrastructure layer.",
        }.into_definition(),
    ]
}

/// Returns all standard built-in guard tests.
#[must_use]
pub fn get_builtin_guard_tests() -> Vec<GuardTestDefinition> {
    let mut tests = Vec::new();
    tests.extend(structural_guards());
    tests.extend(complexity_guards());
    tests.extend(hygiene_guards());
    tests.extend(rust_guards());
    tests
}
