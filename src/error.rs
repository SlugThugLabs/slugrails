//! `SlugRails` error taxonomy.

use std::path::PathBuf;
use thiserror::Error;

/// Granular architecture contract errors.
#[derive(Debug, Error)]
pub enum ContractError {
    /// Missing architecture contract file at the specified path.
    #[error("Missing architecture contract at {path}. Hint: {hint}")]
    MissingContract {
        /// Path where the contract was expected.
        path: PathBuf,
        /// Actionable hint to create or locate the contract.
        hint: String,
    },

    /// Unclosed frontmatter fence in ARCHITECTURE.md.
    #[error("Unclosed '+++' TOML frontmatter fence in ARCHITECTURE.md")]
    UnclosedFrontmatter,

    /// Custom contract error with message.
    #[error("Contract error: {0}")]
    Custom(String),
}

impl From<String> for ContractError {
    fn from(msg: String) -> Self {
        Self::Custom(msg)
    }
}

impl From<&str> for ContractError {
    fn from(msg: &str) -> Self {
        Self::Custom(msg.to_string())
    }
}

/// Granular interactive menu errors.
#[derive(Debug, Error)]
pub enum MenuError {
    /// Standard I/O error during terminal interaction.
    #[error("Terminal I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid menu choice index.
    #[error("Invalid menu choice: {0}")]
    InvalidChoice(usize),

    /// Subprocess execution failure.
    #[error("Subprocess execution failed for '{command}': {reason}")]
    SubprocessFailed {
        /// Executed command name or path.
        command: String,
        /// Reason for failure.
        reason: String,
    },
}

/// Core error type for `SlugRails` operations.
#[derive(Debug, Error)]
pub enum SlugRailsError {
    /// File system I/O error at a specific path.
    #[error("I/O error at {path}: {source}")]
    Io {
        /// Target path where I/O failed.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Sandbox security violation.
    #[error(
        "Sandbox violation: path {path} is unsafe (outside project root or accesses sensitive system paths)"
    )]
    SandboxViolation {
        /// Offending path.
        path: PathBuf,
    },

    /// JSON serialization or deserialization error.
    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML configuration parsing error.
    #[error("TOML parsing error in {path}: {source}")]
    TomlParse {
        /// Path to TOML file.
        path: PathBuf,
        /// Underlying TOML parsing error.
        #[source]
        source: toml::de::Error,
    },

    /// Architecture contract error.
    #[error("Architecture contract error: {0}")]
    Contract(#[from] ContractError),

    /// Interactive terminal menu error.
    #[error("Interactive menu error: {0}")]
    Menu(#[from] MenuError),

    /// Schema validation error for a guard test definition.
    #[error("Guard test schema validation error for {test_id}: {reason}")]
    InvalidGuardTest {
        /// Identifier of the offending guard test.
        test_id: String,
        /// Specific schema validation failure reason.
        reason: String,
    },

    /// Cryptographic exception token verification failure.
    #[error("Exception verification failed: {0}")]
    ExceptionInvalid(String),

    /// Project root not found at the specified path.
    #[error("Project not found at: {0}")]
    ProjectNotFound(PathBuf),

    /// General internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Convenience result type alias for `SlugRails` operations.
pub type Result<T> = std::result::Result<T, SlugRailsError>;
