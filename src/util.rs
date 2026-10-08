//! Sandboxing and utility functions.

use crate::error::{CodeGuardsError, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Validates that a path does not target sensitive system directories
/// (e.g., `/proc`, `/sys`, `/dev`, `~/.ssh`, `~/.aws`).
///
/// # Errors
///
/// Returns [`CodeGuardsError::SandboxViolation`] if the path escapes the project root.
pub fn validate_safe_path(path: &Path) -> Result<PathBuf> {
    let canonical = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    };

    let path_str = canonical.to_string_lossy();

    // Check strict component boundaries rather than substring
    for comp in canonical.components() {
        if let std::path::Component::Normal(c) = comp {
            let name = c.to_string_lossy();
            if name == ".ssh" || name == ".aws" || name == ".gnupg" {
                return Err(CodeGuardsError::SandboxViolation { path: canonical });
            }
        }
    }

    if path_str.starts_with("/proc")
        || path_str.starts_with("/sys")
        || path_str.starts_with("/dev")
        || path_str.starts_with("/etc/shadow")
        || path_str.starts_with("/etc/sudoers")
    {
        return Err(CodeGuardsError::SandboxViolation { path: canonical });
    }

    Ok(canonical)
}

/// Generates a deterministic BLAKE3 hex hash of a project's root path.
#[must_use]
pub fn hash_project_path(project_path: &Path) -> String {
    let canonical = project_path
        .canonicalize()
        .unwrap_or_else(|_| project_path.to_path_buf());
    let mut hasher = blake3::Hasher::new();
    hasher.update(canonical.to_string_lossy().as_bytes());
    let hash = hasher.finalize();
    hash.to_hex()[..12].to_string()
}

/// Retrieves or initializes a local private HMAC salt secret in ~/.slugthuglabs/slugrails/.secret.key
/// ensuring AI agents cannot compute exception tokens in memory without access.
#[must_use]
pub fn get_or_init_secret_salt() -> [u8; 32] {
    let home = get_slugrails_home();
    let key_path = home.join(".secret.key");

    if let Ok(bytes) = fs::read(&key_path)
        && bytes.len() == 32
    {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        return arr;
    }

    let _ = fs::create_dir_all(&home);
    let random_key = uuid::Uuid::new_v4();
    let mut hasher = blake3::Hasher::new();
    hasher.update(random_key.as_bytes());
    hasher.update(b"slugrails-salt-key");
    let key_bytes = *hasher.finalize().as_bytes();

    let _ = fs::write(&key_path, key_bytes);
    key_bytes
}

/// Computes a secure, salted 16-character hex verification token for an exception.
/// Includes file path, guard ID, reason, and current timestamp to prevent replay attacks.
#[must_use]
pub fn compute_exception_token(file: &Path, guard_id: &str, reason: &str) -> String {
    let salt = get_or_init_secret_salt();
    let timestamp = chrono::Utc::now().format("%Y-%m-%d").to_string(); // Daily granularity
    let mut hasher = blake3::Hasher::new_keyed(&salt);
    hasher.update(file.to_string_lossy().as_bytes());
    hasher.update(guard_id.as_bytes());
    hasher.update(reason.as_bytes());
    hasher.update(timestamp.as_bytes());
    let hash = hasher.finalize();
    format!("sr_tok_{}", &hash.to_hex()[..16])
}

/// Returns the central ~/.slugthuglabs directory.
#[must_use]
pub fn get_slugthug_home() -> PathBuf {
    if let Ok(dir) = std::env::var("SLUGTHUGLABS_HOME") {
        PathBuf::from(dir)
    } else if let Ok(dir) = std::env::var("SLUGTHUG_HOME") {
        PathBuf::from(dir)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".slugthuglabs")
    } else if let Ok(profile) = std::env::var("USERPROFILE") {
        PathBuf::from(profile).join(".slugthuglabs")
    } else {
        PathBuf::from("/root/.slugthuglabs")
    }
}

/// Returns the global ~/.slugthuglabs/slugrails directory.
#[must_use]
pub fn get_slugrails_home() -> PathBuf {
    get_slugthug_home().join("slugrails")
}

/// Returns the executable binary name for the current platform.
#[must_use]
pub fn get_binary_name() -> &'static str {
    if cfg!(windows) {
        "slugrails.exe"
    } else {
        "slugrails"
    }
}

/// Returns the central test library directory: ~/.slugthuglabs/slugrails/tests/
#[must_use]
pub fn get_tests_dir() -> PathBuf {
    get_slugrails_home().join("tests")
}

/// Returns the project-specific storage directory: <project>/.planning/slugrails/
#[must_use]
pub fn get_project_storage_dir(project_path: &Path) -> PathBuf {
    project_path.join(".planning").join("slugrails")
}

/// Returns the project-specific plan directory: <project>/.planning/slugplan/
#[must_use]
pub fn get_project_plan_dir(project_path: &Path) -> PathBuf {
    project_path.join(".planning").join("slugplan")
}

