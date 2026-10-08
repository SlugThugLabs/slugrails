//! `CodeGuards` error taxonomy.

use std::path::PathBuf;
use thiserror::Error;

/// Granular architecture contract errors.
#[derive(Debug, Error)]
pub enum ContractError {
    #[error("Missing architecture contract at {path}. Hint: {hint}")]
    MissingContract { path: PathBuf, hint: String },

    #[error("Unclosed '+++' TOML frontmatter fence in ARCHITECTURE.md")]
    UnclosedFrontmatter,

    #[error("Contract error: {0}")]
    Custom(String),
}

impl From<String> for ContractError {
    fn from(s: String) -> Self {
        Self::Custom(s)
    }
}

impl From<&str> for ContractError {
    fn from(s: &str) -> Self {
        Self::Custom(s.to_string())
    }
}

/// Granular interactive menu errors.
#[derive(Debug, Error)]
pub enum MenuError {
    #[error("Terminal I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid menu choice: {0}")]
    InvalidChoice(usize),

    #[error("Subprocess execution failed for '{command}': {reason}")]
    SubprocessFailed { command: String, reason: String },
}

/// Core error type for `CodeGuards` operations.
#[derive(Debug, Error)]
pub enum CodeGuardsError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error(
        "Sandbox violation: path {path} is unsafe (outside project root or accesses sensitive system paths)"
    )]
    SandboxViolation { path: PathBuf },

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parsing error in {path}: {source}")]
    TomlParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("Architecture contract error: {0}")]
    Contract(#[from] ContractError),

    #[error("Interactive menu error: {0}")]
    Menu(#[from] MenuError),

    #[error("Guard test schema validation error for {test_id}: {reason}")]
    InvalidGuardTest { test_id: String, reason: String },

    #[error("Exception verification failed: {0}")]
    ExceptionInvalid(String),

    #[error("Project not found at: {0}")]
    ProjectNotFound(PathBuf),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Convenience result type alias.
pub type Result<T> = std::result::Result<T, CodeGuardsError>;
