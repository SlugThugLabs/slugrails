//! Path normalization, sandbox validation, and directory resolution.

use crate::error::{Result, SlugRailsError};
use std::path::{Component, Path, PathBuf};

/// Reserved Windows DOS device names.
pub const RESERVED_DOS_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Checks if a path or any of its components matches a Windows DOS reserved device name.
#[must_use]
pub fn is_windows_dos_device(path: &Path) -> bool {
    for comp in path.components() {
        if let Component::Normal(c) = comp {
            let name = c.to_string_lossy();
            let upper = name.to_ascii_uppercase();
            let stem = upper
                .split(':')
                .next()
                .unwrap_or("")
                .split('.')
                .next()
                .unwrap_or("");
            if RESERVED_DOS_NAMES.contains(&stem) {
                return true;
            }
        }
    }
    false
}

/// Normalizes a path lexically by resolving `.` and `..` components
/// without accessing the filesystem.
#[must_use]
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => match components.last() {
                Some(Component::Normal(_)) => {
                    components.pop();
                }
                Some(Component::RootDir) => {}
                _ => components.push(comp),
            },
            _ => components.push(comp),
        }
    }
    components.into_iter().collect()
}

/// Validates that a path does not target sensitive system directories
/// (e.g., `/proc`, `/sys`, `/dev`, `~/.ssh`, `~/.aws`).
///
/// # Errors
///
/// Returns [`SlugRailsError::SandboxViolation`] if the path escapes the sandbox or references forbidden resources.
pub fn validate_safe_path(path: &Path) -> Result<PathBuf> {
    let raw_str = path.to_string_lossy();
    if raw_str.starts_with(r"\\") || raw_str.starts_with("//") {
        return Err(SlugRailsError::SandboxViolation {
            path: path.to_path_buf(),
        });
    }
    if raw_str.chars().skip(2).any(|c| c == ':') {
        return Err(SlugRailsError::SandboxViolation {
            path: path.to_path_buf(),
        });
    }

    if is_windows_dos_device(path) {
        return Err(SlugRailsError::SandboxViolation {
            path: path.to_path_buf(),
        });
    }

    let canonical = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    };

    if is_windows_dos_device(&canonical) {
        return Err(SlugRailsError::SandboxViolation { path: canonical });
    }

    let path_str = canonical.to_string_lossy();

    for comp in canonical.components() {
        if matches!(comp, Component::ParentDir) {
            return Err(SlugRailsError::SandboxViolation { path: canonical });
        }
        if let Component::Normal(c) = comp {
            let name = c.to_string_lossy();
            if name == ".ssh" || name == ".aws" || name == ".gnupg" {
                return Err(SlugRailsError::SandboxViolation { path: canonical });
            }
        }
    }

    if path_str.starts_with("/proc")
        || path_str.starts_with("/sys")
        || path_str.starts_with("/dev")
        || path_str.starts_with("/etc/shadow")
        || path_str.starts_with("/etc/sudoers")
    {
        return Err(SlugRailsError::SandboxViolation { path: canonical });
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

/// Returns the project-specific storage directory: `<project>/.planning/slugrails/`
#[must_use]
pub fn get_project_storage_dir(project_path: &Path) -> PathBuf {
    project_path.join(".planning").join("slugrails")
}

/// Returns the project-specific plan directory: `<project>/.planning/slugplan/`
#[must_use]
pub fn get_project_plan_dir(project_path: &Path) -> PathBuf {
    project_path.join(".planning").join("slugplan")
}
