//! Trait definition for modular architecture guard checks.

use super::context::FileContext;
use crate::contract::ArchitectureContract;
use crate::types::Violation;

/// A modular architectural guard check.
pub trait GuardCheck: Send + Sync {
    /// Canonical identifier (e.g. `complexity/source-limits`).
    fn id(&self) -> &'static str;

    /// Evaluates the check against a file context and returns any violations found.
    fn check(&self, ctx: &FileContext, contract: &ArchitectureContract) -> Vec<Violation>;
}
