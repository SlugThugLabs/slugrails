//! `SlugRails` library root.
//!
//! Architectural rail builder and contract scaffolding for software projects
//! and AI coding agents.

#![forbid(unsafe_code)]

pub mod error;
pub mod style;
pub mod types;
pub mod util;

pub use error::{ContractError, MenuError, Result, SlugRailsError};
pub use style::Style;
pub use types::{
    ExceptionEntry, GuardReport, GuardTestDefinition, ScanOutcome, Severity, Span, Violation,
};
