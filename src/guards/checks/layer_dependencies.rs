//! Structural guard: flags imports that cross a declared layer boundary.

use super::exception::has_valid_exception;
use crate::analyzer::extract_imported_modules;
use crate::contract::ArchitectureContract;
use crate::guards::context::FileContext;
use crate::guards::traits::GuardCheck;
use crate::types::{Severity, Violation};

pub struct LayerDependenciesCheck;

impl GuardCheck for LayerDependenciesCheck {
    fn id(&self) -> &'static str {
        "structural/layer-dependencies"
    }

    fn check(&self, ctx: &FileContext<'_>, contract: &ArchitectureContract) -> Vec<Violation> {
        let mut violations = Vec::new();
        check_layer_dependencies(ctx, contract, &mut violations);
        violations
    }
}

/// Enforces the DAG import hierarchy defined in `allowed_dependencies`.
pub fn check_layer_dependencies(
    ctx: &FileContext<'_>,
    contract: &ArchitectureContract,
    violations: &mut Vec<Violation>,
) {
    let imported_modules = extract_imported_modules(ctx.content);
    for (source_layer, allowed) in &contract.allowed_dependencies {
        // Platform-safe component check (handles both Unix '/' and Windows '\')
        let in_source_layer = ctx
            .rel_file
            .components()
            .any(|c| c.as_os_str() == source_layer.as_str());

        if in_source_layer {
            for (line_num, import_path) in &imported_modules {
                for target_layer in contract.allowed_dependencies.keys() {
                    let crosses_boundary = target_layer != source_layer
                        && !allowed.contains(target_layer)
                        && (import_path.contains(&format!("crate::{target_layer}"))
                            || import_path.starts_with(target_layer));

                    if crosses_boundary
                        && !has_valid_exception(
                            ctx.file,
                            &ctx.guard.id,
                            ctx.content,
                            ctx.exceptions,
                        )
                    {
                        violations.push(Violation {
                            guard_id: ctx.guard.id.clone(),
                            file: ctx.rel_file.to_path_buf(),
                            line: Some(*line_num),
                            message: format!(
                                "Illegal import of '{target_layer}' from '{source_layer}' (import: '{import_path}')"
                            ),
                            severity: Severity::Error,
                            fix_suggestion: Some(format!(
                                "Module '{source_layer}' cannot depend on '{target_layer}'. Refactor access through declared layer boundary."
                            )),
                            rule_reference: Some(format!(
                                ".planning/ARCHITECTURE.md [allowed_dependencies: {source_layer}]"
                            )),
                        });
                    }
                }
            }
        }
    }
}
