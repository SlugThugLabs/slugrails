//! Architecture contract parser and validator for .planning/ARCHITECTURE.md.

use crate::error::{CodeGuardsError, ContractError, Result};
use crate::library::catalog::GuardCatalog;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

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

/// Extracts TOML frontmatter delimited by `+++` fences from markdown content.
///
/// # Errors
///
/// Returns [`CodeGuardsError::Contract`] if frontmatter fences are unbalanced,
/// or [`CodeGuardsError::TomlParse`] if the TOML content is malformed.
pub fn parse_frontmatter(content: &str) -> Result<(ArchitectureContract, &str)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("+++") {
        return Ok((ArchitectureContract::default(), content));
    }

    let rest = &trimmed[3..];
    let end_idx = rest
        .find("+++")
        .ok_or_else(|| CodeGuardsError::Contract(ContractError::UnclosedFrontmatter))?;

    let toml_str = &rest[..end_idx];
    let body = &rest[end_idx + 3..];

    let contract: ArchitectureContract =
        toml::from_str(toml_str).map_err(|e| CodeGuardsError::TomlParse {
            path: PathBuf::from(".planning/ARCHITECTURE.md"),
            source: e,
        })?;

    Ok((contract, body))
}

/// Pure in-memory validation of an architecture contract against a known module list and catalog.
#[must_use]
pub fn validate_contract(
    contract: &ArchitectureContract,
    existing_modules: &[String],
    catalog: &GuardCatalog,
) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut ready_guards = Vec::new();
    let mut missing_guards = Vec::new();

    for module in &contract.modules {
        if !existing_modules.contains(module) {
            warnings.push(format!(
                "Declared module '{module}' does not exist on disk under src/"
            ));
        }
    }

    for enforce_item in &contract.enforce {
        if let Some(entry) = catalog.resolve(enforce_item) {
            ready_guards.push(entry.id.clone());
        } else {
            missing_guards.push(enforce_item.clone());
            errors.push(format!(
                "Required guard test '{enforce_item}' is not found in ~/.slugthug/codeguards/tests/. Use create_guard_test to generate it."
            ));
        }
    }

    let is_valid = errors.is_empty();
    ValidationResult {
        is_valid,
        contract: Some(contract.clone()),
        ready_guards,
        missing_guards,
        errors,
        warnings,
    }
}

/// Loads and parses .planning/ARCHITECTURE.md from a project directory.
///
/// # Errors
///
/// Returns [`CodeGuardsError::Contract`] if missing, or [`CodeGuardsError::Io`] if unreadable.
pub fn load_architecture(project_root: &Path) -> Result<ArchitectureContract> {
    let arch_file = project_root.join(".planning").join("ARCHITECTURE.md");
    if !arch_file.exists() {
        return Err(CodeGuardsError::Contract(ContractError::MissingContract {
            path: arch_file,
            hint: "Run 'codeguards-mcp init' to automatically scaffold one for this project."
                .to_string(),
        }));
    }

    let content = fs::read_to_string(&arch_file).map_err(|e| CodeGuardsError::Io {
        path: arch_file,
        source: e,
    })?;

    let (contract, _) = parse_frontmatter(&content)?;
    Ok(contract)
}

/// Validates .planning/ARCHITECTURE.md against disk reality and the guard-test catalog.
///
/// # Errors
///
/// Returns [`CodeGuardsError::Contract`] if ARCHITECTURE.md is missing or malformed.
pub fn validate_architecture(
    project_root: &Path,
    catalog: &GuardCatalog,
) -> Result<ValidationResult> {
    let contract = match load_architecture(project_root) {
        Ok(c) => c,
        Err(e) => {
            return Ok(ValidationResult {
                is_valid: false,
                contract: None,
                ready_guards: Vec::new(),
                missing_guards: Vec::new(),
                errors: vec![format!("Failed to load ARCHITECTURE.md: {e}")],
                warnings: Vec::new(),
            });
        }
    };

    let mut existing_modules = Vec::new();
    let src_dir = project_root.join("src");
    if src_dir.is_dir() {
        for module in &contract.modules {
            let rs_file = src_dir.join(format!("{module}.rs"));
            let mod_dir = src_dir.join(module);
            if rs_file.exists() || mod_dir.exists() {
                existing_modules.push(module.clone());
            }
        }
    }

    Ok(validate_contract(&contract, &existing_modules, catalog))
}
