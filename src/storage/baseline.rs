//! BLAKE3 architecture contract baseline recording and verification.

use crate::error::{CodeGuardsError, ContractError, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Error message emitted when ARCHITECTURE.md has been tampered with.
pub const TAMPER_ERROR_MSG: &str =
    "ARCHITECTURE.md hash does not match approved baseline (.planning/slugrails/contract.baseline). Update baseline with authorized token or slugrails plan.";

impl ContractError {
    /// Constructs an architectural contract malformed error.
    #[allow(non_snake_case)]
    #[must_use]
    pub fn Malformed(msg: &str) -> Self {
        Self::Custom(msg.to_string())
    }
}

/// Returns the path to the project's contract baseline file:
/// `<project_root>/.planning/slugrails/contract.baseline`
#[must_use]
pub fn get_contract_baseline_path(project_root: &Path) -> PathBuf {
    crate::util::get_project_storage_dir(project_root).join("contract.baseline")
}

/// Computes the deterministic BLAKE3 hex hash of the architecture contract content.
#[must_use]
pub fn compute_contract_hash(content: &str) -> String {
    blake3::hash(content.as_bytes()).to_hex().to_string()
}

/// Records the BLAKE3 hash of the contract content to `.planning/slugrails/contract.baseline`.
///
/// # Errors
///
/// Returns [`std::io::Error`] if creating directories or writing the baseline file fails.
pub fn save_contract_baseline(project_root: &Path, content: &str) -> std::io::Result<()> {
    let baseline_path = get_contract_baseline_path(project_root);
    if let Some(parent) = baseline_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let hash = compute_contract_hash(content);
    fs::write(&baseline_path, format!("{hash}\n"))
}

/// Verifies that ARCHITECTURE.md matches `.planning/slugrails/contract.baseline` if present.
///
/// # Errors
///
/// Returns [`CodeGuardsError::Contract`] if the hash does not match,
/// or [`CodeGuardsError::Io`] if reading the baseline file fails.
pub fn verify_contract_baseline(project_root: &Path, content: &str) -> Result<()> {
    let baseline_path = get_contract_baseline_path(project_root);
    if !baseline_path.exists() {
        return Ok(());
    }

    let stored = fs::read_to_string(&baseline_path).map_err(|e| CodeGuardsError::Io {
        path: baseline_path.clone(),
        source: e,
    })?;

    let current = compute_contract_hash(content);
    if stored.trim() != current {
        return Err(CodeGuardsError::Contract(ContractError::Malformed(TAMPER_ERROR_MSG)));
    }

    Ok(())
}
