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

/// Rejects Windows path syntax that can escape a root: UNC paths, alternate data
/// streams, and reserved DOS device names.
fn reject_windows_path_syntax(path: &Path) -> Result<()> {
    let raw_str = path.to_string_lossy();
    let (target_path, check_str) = if let Some(rest) = raw_str.strip_prefix(r"\\?\") {
        let b = rest.as_bytes();
        if b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':' {
            (Path::new(rest), rest)
        } else {
            return Err(SlugRailsError::SandboxViolation {
                path: path.to_path_buf(),
            });
        }
    } else {
        (path, raw_str.as_ref())
    };

    if check_str.starts_with(r"\\") || check_str.starts_with("//") {
        return Err(SlugRailsError::SandboxViolation {
            path: path.to_path_buf(),
        });
    }
    if check_str.chars().skip(2).any(|c| c == ':') {
        return Err(SlugRailsError::SandboxViolation {
            path: path.to_path_buf(),
        });
    }
    if is_windows_dos_device(target_path) {
        return Err(SlugRailsError::SandboxViolation {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

/// Rejects a resolved path that lives in a sensitive system location.
fn reject_sensitive_location(canonical: &Path) -> Result<()> {
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
        if let Component::Normal(c) = comp {
            let name = c.to_string_lossy();
            if name == ".ssh" || name == ".aws" || name == ".gnupg" {
                return Err(SlugRailsError::SandboxViolation {
                    path: canonical.to_path_buf(),
                });
            }
        }
    }
    if canonical.starts_with(Path::new("/proc"))
        || canonical.starts_with(Path::new("/sys"))
        || canonical.starts_with(Path::new("/dev"))
        || canonical.starts_with(Path::new("/etc/shadow"))
        || canonical.starts_with(Path::new("/etc/sudoers"))
    {
        return Err(SlugRailsError::SandboxViolation {
            path: canonical.to_path_buf(),
        });
    }
    Ok(())
}

/// Resolves and validates an authorized project root against the deny-list.
///
/// This checks the *shape* of the root and rejects sensitive locations; it does
/// **not** authorize the caller to choose that root (see IMPLEMENTATION_PROGRESS).
///
/// # Errors
///
/// Returns [`SlugRailsError::SandboxViolation`] for malformed or sensitive roots.
pub fn resolve_root(root: &Path) -> Result<PathBuf> {
    reject_windows_path_syntax(root)?;
    let canonical = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    reject_sensitive_location(&canonical)?;
    Ok(canonical)
}

/// Resolves `path` and requires it to stay inside the authorized `root`.
///
/// The root should be resolved first with [`resolve_root`]. Relative paths are
/// resolved against the root; a non-existent target is resolved lexically (so an
/// exception can be granted before a file exists) and must still stay inside the
/// root. Existing paths are canonicalized, so a symlink that escapes is rejected.
///
/// # Errors
///
/// Returns [`SlugRailsError::SandboxViolation`] if the path escapes the root or
/// references forbidden resources.
pub fn validate_safe_path(root: &Path, path: &Path) -> Result<PathBuf> {
    reject_windows_path_syntax(path)?;
    let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

    let canonical = match path.canonicalize() {
        Ok(resolved) => resolved,
        Err(_) => {
            let normalized = normalize_path(path);
            let joined = if normalized.is_absolute() {
                normalized
            } else {
                canonical_root.join(normalized)
            };
            normalize_path(&joined)
        }
    };

    reject_sensitive_location(&canonical)?;

    if canonical != canonical_root && !canonical.starts_with(&canonical_root) {
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
