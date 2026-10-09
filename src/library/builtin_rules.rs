//! Structural and complexity guard test definitions.

use crate::types::GuardTestDefinition;
use std::collections::BTreeMap;

pub(crate) struct GuardSpec<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub category: &'a str,
    pub summary: &'a str,
    pub tags: &'a [&'a str],
    pub aliases: &'a [&'a str],
    pub engine: &'a str,
    pub params: &'a [(&'a str, serde_json::Value)],
    pub remediation: &'a str,
}

impl GuardSpec<'_> {
    pub(crate) fn into_definition(self) -> GuardTestDefinition {
        let mut default_params = BTreeMap::new();
        for (k, v) in self.params {
            default_params.insert((*k).to_string(), v.clone());
        }
        GuardTestDefinition {
            id: self.id.to_string(),
            name: self.name.to_string(),
            category: self.category.to_string(),
            version: "1.0.0".to_string(),
            summary: self.summary.to_string(),
            tags: self.tags.iter().map(|s| (*s).to_string()).collect(),
            aliases: self.aliases.iter().map(|s| (*s).to_string()).collect(),
            engine: self.engine.to_string(),
            default_params,
            remediation: self.remediation.to_string(),
        }
    }
}

pub(crate) fn structural_guards() -> Vec<GuardTestDefinition> {
    vec![
        GuardSpec {
            id: "structural/docs-drift",
            name: "docs_drift",
            category: "structural",
            summary: "Verifies bidirectional agreement between ARCHITECTURE.md and codebase reality",
            tags: &["architecture", "docs", "drift", "module-map"],
            aliases: &["docs_drift", "architecture_drift"],
            engine: "docs_drift",
            params: &[],
            remediation: "Update .planning/ARCHITECTURE.md or remove unlisted rogue files in src/.",
        }.into_definition(),
        GuardSpec {
            id: "structural/layer-dependencies",
            name: "layer_dependencies",
            category: "structural",
            summary: "Enforces strict DAG import hierarchy based on allowed_dependencies",
            tags: &["layers", "imports", "dependencies", "dag"],
            aliases: &["layer_dependencies", "forbidden_imports", "dependency_guard"],
            engine: "layer_dependencies",
            params: &[],
            remediation: "Refactor imported dependency through its declared public interface or update layer bounds in ARCHITECTURE.md.",
        }.into_definition(),
        GuardSpec {
            id: "structural/manifest-dependencies",
            name: "manifest_dependencies",
            category: "structural",
            summary: "Verifies Cargo.toml / package.json dependency whitelists and layer bounds",
            tags: &["cargo", "manifest", "dependencies"],
            aliases: &["manifest_dependencies", "crate_dependencies"],
            engine: "manifest_dependencies",
            params: &[],
            remediation: "Remove forbidden external crate from Cargo.toml or move logic to the appropriate infrastructure layer.",
        }.into_definition(),
    ]
}

pub(crate) fn complexity_guards() -> Vec<GuardTestDefinition> {
    vec![
        GuardSpec {
            id: "complexity/source-limits",
            name: "source_limits",
            category: "complexity",
            summary: "Enforces strict production code line ceiling per file (default 400 lines)",
            tags: &["loc", "lines", "file-size", "small-files"],
            aliases: &["source_limits", "file_size_guard", "small_files_limit", "max_file_lines"],
            engine: "source_limits",
            params: &[("max_lines", serde_json::json!(400))],
            remediation: "Split large module into focused cohesive submodules under a directory module.",
        }.into_definition(),
        GuardSpec {
            id: "complexity/function-limits",
            name: "function_limits",
            category: "complexity",
            summary: "Enforces maximum function/method length (default 80 lines)",
            tags: &["functions", "lines", "complexity"],
            aliases: &["function_limits", "max_function_lines"],
            engine: "function_limits",
            params: &[("max_lines", serde_json::json!(80))],
            remediation: "Decompose long function into smaller single-responsibility helper functions.",
        }.into_definition(),
        GuardSpec {
            id: "complexity/deep-nesting",
            name: "deep_nesting",
            category: "complexity",
            summary: "Flags functions exceeding maximum control flow nesting depth (default 4)",
            tags: &["nesting", "complexity", "depth", "cyclomatic"],
            aliases: &["deep_nesting", "max_nesting_depth"],
            engine: "deep_nesting",
            params: &[("max_depth", serde_json::json!(4))],
            remediation: "Flatten control flow using early returns, guard clauses, or extracted helper functions.",
        }.into_definition(),
    ]
}
