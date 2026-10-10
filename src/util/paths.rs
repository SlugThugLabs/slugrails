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
        if let Component::Normal(comp_name) = comp {
            let name_lossy = comp_name.to_string_lossy();
            let upper_name = name_lossy.to_ascii_uppercase();
            let stem = upper_name
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

fn validate_raw_syntax(path: &Path, raw_str: &str) -> Result<()> {
    if raw_str.starts_with(r"\\") || raw_str.starts_with("//") {
        return Err(SlugRailsError::SandboxViolation {
            path: path.to_path_buf(),
        });
    }
    if raw_str.chars().skip(2).any(|char_val| char_val == ':') {
        return Err(SlugRailsError::SandboxViolation {
            path: path.to_path_buf(),
        });
    }
    if is_windows_dos_device(path) {
        return Err(SlugRailsError::SandboxViolation {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

fn validate_canonical_components(canonical: &Path) -> Result<()> {
    if is_windows_dos_device(canonical) {
        return Err(SlugRailsError::SandboxViolation {
            path: canonical.to_path_buf(),
        });
    }
    for comp in canonical.components() {
        if matches!(comp, Component::ParentDir) {
            return Err(SlugRailsError::SandboxViolation {
                path: canonical.to_path_buf(),
            });
        }
        if let Component::Normal(comp_name) = comp {
            let name = comp_name.to_string_lossy();
            if name == ".ssh" || name == ".aws" || name == ".gnupg" {
                return Err(SlugRailsError::SandboxViolation {
                    path: canonical.to_path_buf(),
                });
            }
        }
    }
    Ok(())
}

fn validate_system_prefixes(canonical: &Path, path_str: &str) -> Result<()> {
    if path_str.starts_with("/proc")
        || path_str.starts_with("/sys")
        || path_str.starts_with("/dev")
        || path_str.starts_with("/etc/shadow")
        || path_str.starts_with("/etc/sudoers")
    {
        return Err(SlugRailsError::SandboxViolation {
            path: canonical.to_path_buf(),
        });
    }
    Ok(())
}

/// Validates that a path does not target sensitive system directories
/// (e.g., `/proc`, `/sys`, `/dev`, `~/.ssh`, `~/.aws`).
///
/// # Errors
///
/// Returns [`SlugRailsError::SandboxViolation`] if the path escapes the sandbox or references forbidden resources.
pub fn validate_safe_path(path: &Path) -> Result<PathBuf> {
    let raw_str = path.to_string_lossy();
    validate_raw_syntax(path, &raw_str)?;

    let canonical = match path.canonicalize() {
        Ok(resolved) => resolved,
        Err(_) => path.to_path_buf(),
    };

    validate_canonical_components(&canonical)?;
    let path_str = canonical.to_string_lossy();
    validate_system_prefixes(&canonical, &path_str)?;

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
