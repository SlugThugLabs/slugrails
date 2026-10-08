//! Built-in guard test definitions embedded directly into the binary.

use crate::types::GuardTestDefinition;
use std::collections::BTreeMap;

#[inline]
fn guard(
    id: &str,
    name: &str,
    category: &str,
    summary: &str,
    tags: &[&str],
    aliases: &[&str],
    engine: &str,
    params: &[(&str, serde_json::Value)],
    remediation: &str,
) -> GuardTestDefinition {
    let mut default_params = BTreeMap::new();
    for (k, v) in params {
        default_params.insert((*k).to_string(), v.clone());
    }
    GuardTestDefinition {
        id: id.to_string(),
        name: name.to_string(),
        category: category.to_string(),
        version: "1.0.0".to_string(),
        summary: summary.to_string(),
        tags: tags.iter().map(|s| (*s).to_string()).collect(),
        aliases: aliases.iter().map(|s| (*s).to_string()).collect(),
        engine: engine.to_string(),
        default_params,
        remediation: remediation.to_string(),
    }
}

/// Returns all standard built-in guard tests.
#[must_use]
pub fn get_builtin_guard_tests() -> Vec<GuardTestDefinition> {
    vec![
        guard(
            "structural/docs-drift", "docs_drift", "structural",
            "Verifies bidirectional agreement between ARCHITECTURE.md and codebase reality",
            &["architecture", "docs", "drift", "module-map"],
            &["docs_drift", "architecture_drift"],
            "docs_drift", &[],
            "Update .planning/ARCHITECTURE.md or remove unlisted rogue files in src/.",
        ),
        guard(
            "structural/layer-dependencies", "layer_dependencies", "structural",
            "Enforces strict DAG import hierarchy based on allowed_dependencies",
            &["layers", "imports", "dependencies", "dag"],
            &["layer_dependencies", "forbidden_imports", "dependency_guard"],
            "layer_dependencies", &[],
            "Refactor imported dependency through its declared public interface or update layer bounds in ARCHITECTURE.md.",
        ),
        guard(
            "structural/manifest-dependencies", "manifest_dependencies", "structural",
            "Verifies Cargo.toml / package.json dependency whitelists and layer bounds",
            &["cargo", "manifest", "dependencies"],
            &["manifest_dependencies", "crate_dependencies"],
            "manifest_dependencies", &[],
            "Remove forbidden external crate from Cargo.toml or move logic to the appropriate infrastructure layer.",
        ),
        guard(
            "complexity/source-limits", "source_limits", "complexity",
            "Enforces strict production code line ceiling per file (default 400 lines)",
            &["loc", "lines", "file-size", "small-files"],
            &["source_limits", "file_size_guard", "small_files_limit", "max_file_lines"],
            "source_limits", &[("max_lines", serde_json::json!(400))],
            "Split large module into focused cohesive submodules under a directory module.",
        ),
        guard(
            "complexity/function-limits", "function_limits", "complexity",
            "Enforces maximum function/method length (default 80 lines)",
            &["functions", "lines", "complexity"],
            &["function_limits", "max_function_lines"],
            "function_limits", &[("max_lines", serde_json::json!(80))],
            "Decompose long function into smaller single-responsibility helper functions.",
        ),
        guard(
            "quality/no-stubs", "no_stubs", "quality",
            "Rejects unimplemented stub macros (todo!, unimplemented!) in production",
            &["stubs", "todo", "quality"],
            &["no_stubs", "check_no_stubs", "forbid_stubs"],
            "no_stubs", &[],
            "Provide complete production implementation or remove unused stub.",
        ),
        guard(
            "hygiene/no-secrets", "no_secrets", "hygiene",
            "Detects hardcoded API keys, private tokens, and credentials",
            &["security", "secrets", "tokens", "keys"],
            &["no_secrets", "no_hardcoded_secrets", "secret_scanner"],
            "no_secrets", &[],
            "Extract hardcoded secret into environment variable or configuration file.",
        ),
        guard(
            "hygiene/no-duplicates", "no_duplicates", "hygiene",
            "Disallows duplicate test function names and duplicated git commit subjects",
            &["dedupe", "tests", "git"],
            &["no_duplicates", "check_no_duplicates"],
            "no_duplicates", &[],
            "Rename duplicate test function to describe specific unique behavior.",
        ),
        guard(
            "hygiene/no-debug-prints", "no_debug_prints", "hygiene",
            "Rejects leftover debugging statements in production",
            &["logging", "debug", "prints"],
            &["no_debug_prints", "debug_statements"],
            "no_debug_prints", &[],
            "Replace debug print statement with structured tracing or remove it before committing.",
        ),
        guard(
            "languages/rust/no-unwrap", "no_unwrap", "languages/rust",
            "Prohibits bare .unwrap() and .expect() calls in production Rust source code",
            &["rust", "errors", "unwrap", "safety"],
            &["no_unwrap", "forbid_unwrap"],
            "rust_no_unwrap", &[],
            "Use ? error propagation with thiserror or return a structured Result<T, E>.",
        ),
        guard(
            "languages/rust/tracing-instrument", "tracing_instrument", "languages/rust",
            "Requires #[tracing::instrument] on public API functions",
            &["rust", "tracing", "observability"],
            &["tracing_instrument", "require_tracing"],
            "rust_tracing_instrument", &[],
            "Add #[tracing::instrument] above public entrypoint function.",
        ),
        guard(
            "languages/rust/unsafe-policy", "unsafe_policy", "languages/rust",
            "Enforces #![forbid(unsafe_code)] or strict bounds on unsafe blocks",
            &["rust", "unsafe", "security"],
            &["unsafe_policy", "forbid_unsafe"],
            "rust_unsafe_policy", &[],
            "Remove unsafe block or add crate-level #![forbid(unsafe_code)] directive.",
        ),
        guard(
            "languages/rust/runtime-leak", "runtime_leak", "languages/rust",
            "Prevents async runtime crates (tokio, wasmtime, rayon) from leaking into pure domain/contracts",
            &["rust", "tokio", "leak", "architecture"],
            &["runtime_leak", "tokio_leak_guard"],
            "rust_runtime_leak", &[],
            "Isolate async runtime calls into the transport or infrastructure layer.",
        ),
    ]
}
