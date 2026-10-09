//! Fast, .gitignore-aware parallel file walker using Rayon and Ignore crates.

use crate::error::{CodeGuardsError, Result};
use ignore::{WalkBuilder, WalkState};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};

/// Maximum file size (2 MB) scanned into memory to protect against buffer exhaustion.
pub const MAX_FILE_SIZE_BYTES: u64 = 2 * 1024 * 1024;

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

const MAX_DIRECTORY_DEPTH: usize = 32;
const MAX_SCANNED_FILES: usize = 50_000;

/// Collects all relevant source files in the project root.
///
/// # Errors
///
/// Returns [`CodeGuardsError::Io`] if the project directory cannot be traversed.
pub fn collect_source_files(project_root: &Path) -> Result<Vec<PathBuf>> {
    if let Err(e) = std::fs::metadata(project_root) {
        return Err(CodeGuardsError::Io {
            path: project_root.to_path_buf(),
            source: e,
        });
    }

    let (tx, rx) = mpsc::channel();
    let count = Arc::new(AtomicUsize::new(0));
    let budget_warned = Arc::new(AtomicBool::new(false));

    let walker = WalkBuilder::new(project_root)
        .standard_filters(true)
        .hidden(true)
        .parents(true)
        .max_depth(Some(MAX_DIRECTORY_DEPTH))
        .build_parallel();

    walker.run(|| {
        let tx = tx.clone();
        let count = Arc::clone(&count);
        let budget_warned = Arc::clone(&budget_warned);

        Box::new(move |result| {
            if count.load(Ordering::Relaxed) >= MAX_SCANNED_FILES {
                return WalkState::Quit;
            }

            let entry = match result {
                Ok(entry) => entry,
                Err(err) => {
                    tracing::debug!(error = %err, "Skipping unreadable entry during traversal");
                    return WalkState::Continue;
                }
            };

            let path = entry.path();
            if entry.file_type().is_some_and(|ft| ft.is_dir()) {
                if entry.depth() > 0 && is_path_ignored(path) {
                    return WalkState::Skip;
                }
                return WalkState::Continue;
            }

            if path.is_file()
                && let Some(ext) = path.extension().and_then(|e| e.to_str())
                && matches!(ext, "rs" | "py" | "ts" | "js" | "go" | "toml")
            {
                if is_path_ignored(path) {
                    return WalkState::Continue;
                }
                if let Ok(meta) = entry.metadata() {
                    if meta.len() > MAX_FILE_SIZE_BYTES {
                        tracing::warn!(
                            path = %path.display(),
                            size = meta.len(),
                            "Skipping file exceeding 2MB memory defense limit"
                        );
                        return WalkState::Continue;
                    }
                }
                if count.fetch_add(1, Ordering::Relaxed) >= MAX_SCANNED_FILES {
                    if !budget_warned.swap(true, Ordering::Relaxed) {
                        tracing::warn!(
                            "Scan reached maximum file budget of {MAX_SCANNED_FILES} files"
                        );
                    }
                    return WalkState::Quit;
                }
                if tx.send(path.to_path_buf()).is_err() {
                    return WalkState::Quit;
                }
            }

            WalkState::Continue
        })
    });

    drop(tx);
    let mut files: Vec<PathBuf> = rx.into_iter().collect();
    if files.len() > MAX_SCANNED_FILES {
        files.truncate(MAX_SCANNED_FILES);
    }
    files.sort();
    Ok(files)
}

/// Collects only files modified in the active git worktree.
///
/// # Errors
///
/// Returns [`CodeGuardsError::Io`] if git commands fail to execute or\
/// if the project directory cannot be traversed as a fallback.
pub fn collect_git_diff_files(project_root: &Path) -> Result<Vec<PathBuf>> {
    if which::which("git").is_err() {
        tracing::debug!("'git' CLI not found on PATH; falling back to full repository scan");
        return collect_source_files(project_root);
    }

    let output = std::process::Command::new("git")
        .args(["-c", "core.fsmonitor=", "-c", "core.hooksPath=/dev/null", "status", "--porcelain"])
        .current_dir(project_root)
        .output();

    let Ok(output) = output else {
        tracing::debug!("'git status' command failed; falling back to full repository scan");
        return collect_source_files(project_root);
    };

    let mut files = Vec::new();
    let text = String::from_utf8_lossy(&output.stdout);

    for line in text.lines() {
        if line.len() > 3 {
            let raw_rel = line[3..].trim();
            let target_rel = if let Some((_old, new)) = raw_rel.split_once("->") {
                new.trim()
            } else {
                raw_rel
            };
            let clean_rel = target_rel.trim_matches('"');
            let full_path = project_root.join(clean_rel);
            if full_path.is_file()
                && !is_path_ignored(&full_path)
                && let Some(ext) = full_path.extension().and_then(|e| e.to_str())
                && matches!(ext, "rs" | "py" | "ts" | "js" | "go" | "toml")
            {
                if let Ok(meta) = full_path.metadata() {
                    if meta.len() > MAX_FILE_SIZE_BYTES {
                        continue;
                    }
                }
                files.push(full_path);
            }
        }
    }

    Ok(files)
}
