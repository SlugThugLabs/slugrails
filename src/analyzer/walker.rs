//! Fast, .gitignore-aware parallel file walker using Rayon and Ignore crates.

use crate::error::{CodeGuardsError, Result};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

const DEFAULT_IGNORED_DIRS: &[&str] = &[
    ".planning",
    ".git",
    "target",
    "node_modules",
    "legacy-python",
    "sample_code",
    ".venv",
    "venv",
    "env",
    "dist",
    "build",
    ".next",
    ".nuxt",
    ".cache",
    "__pycache__",
];

fn is_path_ignored(path: &Path) -> bool {
    path.components().any(|c| {
        c.as_os_str()
            .to_str()
            .is_some_and(|s| DEFAULT_IGNORED_DIRS.contains(&s))
    })
}

/// Collects all relevant source files in the project root.
///
/// # Errors
///
/// Returns [`CodeGuardsError::Io`] if the project directory cannot be traversed.
pub fn collect_source_files(project_root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let walker = WalkBuilder::new(project_root)
        .standard_filters(true)
        .hidden(true)
        .parents(true)
        .build();

    for result in walker {
        let entry = result.map_err(|e| CodeGuardsError::Io {
            path: project_root.to_path_buf(),
            source: std::io::Error::other(e),
        })?;

        let path = entry.path();
        if path.is_file()
            && let Some(ext) = path.extension().and_then(|e| e.to_str())
            && matches!(ext, "rs" | "py" | "ts" | "js" | "go" | "toml")
        {
            if is_path_ignored(path) {
                continue;
            }
            files.push(path.to_path_buf());
        }
    }

    Ok(files)
}

/// Collects only files modified in the active git worktree.
///
/// # Errors
///
/// Returns [`CodeGuardsError::Io`] if git commands fail to execute or\
/// if the project directory cannot be traversed as a fallback.
pub fn collect_git_diff_files(project_root: &Path) -> Result<Vec<PathBuf>> {
    let output = std::process::Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(project_root)
        .output();

    let Ok(output) = output else {
        return collect_source_files(project_root);
    };

    let mut files = Vec::new();
    let text = String::from_utf8_lossy(&output.stdout);

    for line in text.lines() {
        if line.len() > 3 {
            let rel_path = line[3..].trim();
            let full_path = project_root.join(rel_path);
            if full_path.is_file()
                && !is_path_ignored(&full_path)
                && let Some(ext) = full_path.extension().and_then(|e| e.to_str())
                && matches!(ext, "rs" | "py" | "ts" | "js" | "go" | "toml")
            {
                files.push(full_path);
            }
        }
    }

    if files.is_empty() {
        collect_source_files(project_root)
    } else {
        Ok(files)
    }
}
