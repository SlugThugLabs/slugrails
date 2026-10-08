//! Registry of all available GuardCheck implementations.

use super::checks::{
    DebugPrintsCheck, DeepNestingCheck, FunctionLimitsCheck, LayerDependenciesCheck, NoPanicCheck,
    NoSecretsCheck, NoStubsCheck, NoUnwrapCheck, SourceLimitsCheck, SwallowedErrorsCheck,
    UnsafePolicyCheck,
};
use super::traits::GuardCheck;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Global registry of all built-in executable guard checks.
pub struct GuardRegistry {
    checks: HashMap<&'static str, Box<dyn GuardCheck>>,
}

impl GuardRegistry {
    #[must_use]
    pub fn new() -> Self {
        let mut checks: HashMap<&'static str, Box<dyn GuardCheck>> = HashMap::new();

        let all_checks: Vec<Box<dyn GuardCheck>> = vec![
            Box::new(SourceLimitsCheck),
            Box::new(FunctionLimitsCheck),
            Box::new(DeepNestingCheck),
            Box::new(NoUnwrapCheck),
            Box::new(UnsafePolicyCheck),
            Box::new(NoPanicCheck),
            Box::new(DebugPrintsCheck),
            Box::new(NoStubsCheck),
            Box::new(SwallowedErrorsCheck),
            Box::new(NoSecretsCheck),
            Box::new(LayerDependenciesCheck),
        ];

        for check in all_checks {
            checks.insert(check.id(), check);
        }

        Self { checks }
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<&dyn GuardCheck> {
        self.checks.get(id).map(AsRef::as_ref)
    }

    #[must_use]
    pub fn is_implemented(&self, id: &str) -> bool {
        self.checks.contains_key(id)
    }

    #[must_use]
    pub fn implemented_ids(&self) -> Vec<&'static str> {
        let mut ids: Vec<&'static str> = self.checks.keys().copied().collect();
        ids.sort_unstable();
        ids
    }
}

impl Default for GuardRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Static singleton reference to the guard registry.
pub static DEFAULT_REGISTRY: LazyLock<GuardRegistry> = LazyLock::new(GuardRegistry::new);
