pub mod discovery;
pub mod scaffold;
pub mod types;

pub use crate::storage::baseline::{save_contract_baseline, verify_contract_baseline};
pub use discovery::{ProjectInspection, inspect_project};
pub use scaffold::{plan_architecture, scaffold_tests_harness};
pub use types::{ArchitectureContract, InvariantDeclaration, ValidationResult};

use crate::error::{ContractError, Result, SlugRailsError};
use crate::library::catalog::GuardCatalog;
use std::fs;
use std::path::{Path, PathBuf};

/// Extracts TOML frontmatter delimited by `+++` fences from markdown content.
///
/// # Errors
///
/// Returns [`SlugRailsError::Contract`] if frontmatter fences are unbalanced,
/// or [`SlugRailsError::TomlParse`] if the TOML content is malformed.
pub fn parse_frontmatter(content: &str) -> Result<(ArchitectureContract, &str)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("+++") {
        return Ok((ArchitectureContract::default(), content));
    }

    let rest = &trimmed[3..];
    let end_idx = rest
        .find("+++")
        .ok_or(SlugRailsError::Contract(ContractError::UnclosedFrontmatter))?;

    let toml_str = &rest[..end_idx];
    let body = &rest[end_idx + 3..];

    let contract: ArchitectureContract =
        toml::from_str(toml_str).map_err(|e| SlugRailsError::TomlParse {
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
        } else if let Some(b) = crate::library::builtins::get_builtin_guard_tests()
            .into_iter()
            .find(|b| {
                b.id == *enforce_item || b.name == *enforce_item || b.aliases.contains(enforce_item)
            })
        {
            ready_guards.push(b.id);
        } else {
            missing_guards.push(enforce_item.clone());
            errors.push(format!(
                "Required guard test '{enforce_item}' is not found in ~/.slugthuglabs/slugrails/tests/. Use create_guard_test to generate it."
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
/// Returns [`SlugRailsError::Contract`] if missing, or [`SlugRailsError::Io`] if unreadable.
pub fn load_architecture(project_root: &Path) -> Result<ArchitectureContract> {
    let arch_file = project_root.join(".planning").join("ARCHITECTURE.md");
    if !arch_file.exists() {
        return Err(SlugRailsError::Contract(ContractError::MissingContract {
            path: arch_file,
            hint: "Run 'slugrails init' to automatically scaffold one for this project."
                .to_string(),
        }));
    }

    let content = fs::read_to_string(&arch_file).map_err(|e| SlugRailsError::Io {
        path: arch_file,
        source: e,
    })?;

    crate::storage::baseline::verify_contract_baseline(project_root, &content)?;

    let (contract, _) = parse_frontmatter(&content)?;
    Ok(contract)
}

/// Validates .planning/ARCHITECTURE.md against disk reality and the guard-test catalog.
///
/// # Errors
///
/// Returns [`SlugRailsError::Contract`] if ARCHITECTURE.md is missing or malformed.
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
        for m in &contract.modules {
            if src_dir.join(format!("{m}.rs")).exists() || src_dir.join(m).exists() {
                existing_modules.push(m.clone());
            }
        }
    }

    Ok(validate_contract(&contract, &existing_modules, catalog))
}
