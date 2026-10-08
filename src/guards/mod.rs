//! Architectural and code quality guard checkers.

pub mod checks;
pub mod context;
pub mod registry;
pub mod runner;
pub mod traits;

pub use registry::{DEFAULT_REGISTRY, GuardRegistry};
pub use runner::run_guard_checks;
pub use traits::GuardCheck;
