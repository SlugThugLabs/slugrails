//! Architectural contract data types and declarations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A formal behavioral invariant declared in the architecture contract.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct InvariantDeclaration {
    pub id: String,
    #[serde(default)]
    pub component: String,
    pub contract: String,
    #[serde(default)]
    pub enforcement: String,
}

/// TOML frontmatter extracted from .planning/ARCHITECTURE.md.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct ArchitectureContract {
    #[serde(default)]
    pub modules: Vec<String>,
    #[serde(default)]
    pub layers: Vec<String>,
    #[serde(default)]
    pub allowed_dependencies: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub enforce: Vec<String>,
    #[serde(default)]
    pub guard_settings: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub invariants: Vec<InvariantDeclaration>,
}

/// Validation verdict returned by `validate_architecture`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub contract: Option<ArchitectureContract>,
    pub ready_guards: Vec<String>,
    pub missing_guards: Vec<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
