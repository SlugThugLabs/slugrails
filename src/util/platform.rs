//! Platform-specific and cross-platform environment path resolution.

use std::path::PathBuf;

/// Returns the central ~/.slugthuglabs directory.
///
/// Inspects `SLUGTHUGLABS_HOME`, `SLUGTHUG_HOME`, `HOME`, and `USERPROFILE`,
/// falling back to the system temporary directory if none are available.
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
        std::env::temp_dir().join(".slugthuglabs")
    }
}

/// Returns the global ~/.slugthuglabs/slugrails directory.
///
/// Inspects `SLUGRAILS_HOME`, falling back to `get_slugthug_home().join("slugrails")`.
#[must_use]
pub fn get_slugrails_home() -> PathBuf {
    if let Ok(dir) = std::env::var("SLUGRAILS_HOME") {
        PathBuf::from(dir)
    } else {
        get_slugthug_home().join("slugrails")
    }
}

/// Returns the global ~/.slugthuglabs/codeguards directory.
///
/// Inspects `CODEGUARDS_HOME`, falling back to `get_slugthug_home().join("codeguards")`.
#[must_use]
pub fn get_codeguards_home() -> PathBuf {
    if let Ok(dir) = std::env::var("CODEGUARDS_HOME") {
        PathBuf::from(dir)
    } else {
        get_slugthug_home().join("codeguards")
    }
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
