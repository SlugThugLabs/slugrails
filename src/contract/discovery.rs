//! Architectural term discovery and codebase inspection engine.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::path::Path;

/// Result of inspecting a project's disk reality and architectural posture.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProjectInspection {
    pub detected_language: String,
    pub existing_modules: Vec<String>,
    pub candidate_layers: Vec<String>,
    pub allowed_dependencies: BTreeMap<String, Vec<String>>,
    pub recommended_rules: Vec<String>,
    pub interview_prompts: Vec<String>,
    pub candidate_contract: String,
}

/// Inspects a project directory to discover existing modules and formulate candidate rails.
#[must_use]
pub fn inspect_project(root: &Path, intent: &str, style_hint: Option<&str>) -> ProjectInspection {
    let lang = detect_language(root);
    let modules = discover_modules(root);

    let (layers, deps) = match style_hint.unwrap_or("layered").to_lowercase().as_str() {
        "hexagonal" | "ports-and-adapters" => (
            vec![
                "domain".into(),
                "ports".into(),
                "adapters".into(),
                "driver".into(),
            ],
            BTreeMap::from([
                ("adapters".into(), vec!["ports".into(), "domain".into()]),
                ("driver".into(), vec!["adapters".into(), "ports".into()]),
            ]),
        ),
        "cli" | "tool" => (
            vec!["domain".into(), "engine".into(), "cli".into()],
            BTreeMap::from([
                ("cli".into(), vec!["engine".into(), "domain".into()]),
                ("engine".into(), vec!["domain".into()]),
            ]),
        ),
        _ => (
            vec![
                "domain".into(),
                "application".into(),
                "infrastructure".into(),
            ],
            BTreeMap::from([
                ("application".into(), vec!["domain".into()]),
                (
                    "infrastructure".into(),
                    vec!["application".into(), "domain".into()],
                ),
            ]),
        ),
    };

    let mut rules = vec![
        "complexity/source-limits".into(),
        "complexity/function-limits".into(),
        "complexity/deep-nesting".into(),
        "quality/no-stubs".into(),
        "hygiene/swallowed-errors".into(),
        "hygiene/no-secrets".into(),
        "hygiene/no-debug-prints".into(),
    ];
    if lang == "rust" {
        rules.push("languages/rust/no-unwrap".into());
        rules.push("languages/rust/unsafe-policy".into());
    }

    let prompts = vec![
        "What are the core domain entities vs external adapters (databases, APIs, network)?".into(),
        "What are the non-negotiable failure modes—should functions fail-closed with Result/Error?"
            .into(),
        "Should strict file-size limits (<= 200 LOC) apply to enforce high modularity?".into(),
        "Are there special prohibited dependencies (e.g. infrastructure cannot touch UI)?".into(),
    ];

    let contract = render_candidate_contract(&layers, &deps, &rules, intent);

    ProjectInspection {
        detected_language: lang,
        existing_modules: modules,
        candidate_layers: layers,
        allowed_dependencies: deps,
        recommended_rules: rules,
        interview_prompts: prompts,
        candidate_contract: contract,
    }
}

fn detect_language(root: &Path) -> String {
    if root.join("Cargo.toml").exists() {
        "rust".into()
    } else if root.join("package.json").exists() {
        "typescript".into()
    } else if root.join("pyproject.toml").exists() || root.join("requirements.txt").exists() {
        "python".into()
    } else if root.join("go.mod").exists() {
        "go".into()
    } else {
        "polyglot".into()
    }
}

fn discover_modules(root: &Path) -> Vec<String> {
    let mut modules = Vec::new();
    let src_dir = root.join("src");
    let search_dir = if src_dir.is_dir() { &src_dir } else { root };

    if let Ok(entries) = fs::read_dir(search_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir()
                && let Some(name) = path.file_name().and_then(|n| n.to_str())
                && !name.starts_with('.')
                && name != "target"
                && name != "node_modules"
            {
                modules.push(name.to_string());
            }
        }
    }
    modules.sort();
    modules
}

fn render_candidate_contract(
    layers: &[String],
    deps: &BTreeMap<String, Vec<String>>,
    rules: &[String],
    intent: &str,
) -> String {
    let layers_fmt = layers
        .iter()
        .map(|l| format!("\"{l}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let rules_fmt = rules
        .iter()
        .map(|r| format!("\"{r}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let mut deps_fmt = String::new();
    for (k, v) in deps {
        let targets = v
            .iter()
            .map(|t| format!("\"{t}\""))
            .collect::<Vec<_>>()
            .join(", ");
        if let Err(e) = writeln!(deps_fmt, "{k} = [{targets}]") {
            tracing::warn!("Failed to format dependency rule for '{k}': {e}");
        }
    }

    format!(
        "+++\n\
         layers = [{layers_fmt}]\n\
         enforce = [{rules_fmt}]\n\n\
         [allowed_dependencies]\n\
         {deps_fmt}\
         +++\n\n\
         # Intent & Architectural Contract\n\n\
         {intent}\n"
    )
}
