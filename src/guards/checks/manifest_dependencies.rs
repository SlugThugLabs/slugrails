//! Structural guard: verifies Cargo.toml / package.json dependency bounds.

use super::exception::has_valid_exception;
use crate::analyzer::manifest::{scan_cargo_manifest, scan_package_json};
use crate::contract::ArchitectureContract;
use crate::guards::context::FileContext;
use crate::guards::traits::GuardCheck;
use crate::types::{Severity, Violation};

pub struct ManifestDependenciesCheck;

impl GuardCheck for ManifestDependenciesCheck {
    fn id(&self) -> &'static str {
        "structural/manifest-dependencies"
    }

    fn check(&self, ctx: &FileContext<'_>, contract: &ArchitectureContract) -> Vec<Violation> {
        let mut violations = Vec::new();
        check_manifest_dependencies(ctx, contract, &mut violations);
        violations
    }
}

/// Verifies dependency whitelists and declared boundaries in manifests.
pub fn check_manifest_dependencies(
    ctx: &FileContext<'_>,
    contract: &ArchitectureContract,
    violations: &mut Vec<Violation>,
) {
    let file_name = ctx
        .rel_file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if file_name != "Cargo.toml" && file_name != "package.json" {
        return;
    }

    let manifest_deps = if file_name == "Cargo.toml" {
        let root = ctx.file.parent().unwrap_or(ctx.file);
        scan_cargo_manifest(root)
    } else {
        let root = ctx.file.parent().unwrap_or(ctx.file);
        scan_package_json(root)
    };

    let Some(manifest) = manifest_deps else {
        return;
    };

    // If contract specifies forbidden dependencies in guard_settings:
    if let Some(forbidden) = contract
        .guard_settings
        .get("manifest_dependencies")
        .or_else(|| {
            contract
                .guard_settings
                .get("structural/manifest-dependencies")
        })
        .and_then(|v| v.get("forbidden"))
        .and_then(serde_json::Value::as_array)
    {
        for item in forbidden {
            if let Some(crate_name) = item.as_str()
                && manifest.contains(crate_name)
                && !has_valid_exception(ctx.file, &ctx.guard.id, ctx.content, ctx.exceptions)
            {
                violations.push(Violation {
                    guard_id: ctx.guard.id.clone(),
                    file: ctx.rel_file.to_path_buf(),
                    line: Some(1),
                    message: format!("Forbidden manifest dependency detected: '{crate_name}'"),
                    severity: Severity::Error,
                    fix_suggestion: Some(format!(
                        "Remove '{crate_name}' from {file_name} or isolate in separate workspace."
                    )),
                    rule_reference: Some(format!(
                        ".planning/ARCHITECTURE.md [enforce: {}]",
                        ctx.rule_name
                    )),
                });
            }
        }
    }
}
