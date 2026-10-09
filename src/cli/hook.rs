//! Git hook installation and management.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HOOK_SCRIPT: &str = r#"#!/usr/bin/env sh
# SlugRails Pre-Commit Enforcement Gate
if command -v slugrails >/dev/null 2>&1; then
    exec slugrails check
elif command -v slugrails-mcp >/dev/null 2>&1; then
    exec slugrails-mcp check
else
    echo "Error: slugrails not found in PATH. Architectural rails cannot be verified." >&2
    echo "Please install slugrails or commit with --no-verify if explicitly authorized." >&2
    exit 1
fi
"#;

/// Entry point for `slugrails hook` subcommand.
pub fn run_hook<I: Iterator<Item = String>>(mut args: I) -> ExitCode {
    let sub = args.next().unwrap_or_else(|| "install".to_string());
    match sub.as_str() {
        "install" => install_hook(Path::new(".")),
        "uninstall" => uninstall_hook(Path::new(".")),
        _ => {
            eprintln!("Usage: slugrails hook [install|uninstall]");
            ExitCode::from(2)
        }
    }
}

/// Installs the SlugRails pre-commit hook into `.git/hooks/pre-commit`.
pub fn install_hook(repo_root: &Path) -> ExitCode {
    let git_dir = repo_root.join(".git");
    if !git_dir.exists() {
        eprintln!("Error: Not a git repository (missing .git directory).");
        return ExitCode::from(1);
    }

    let hooks_dir = if git_dir.is_dir() {
        git_dir.join("hooks")
    } else {
        // Handle git worktree or submodule where .git is a file
        match resolve_gitdir(&git_dir) {
            Some(dir) => dir.join("hooks"),
            None => {
                eprintln!("Error: Unable to resolve gitdir for worktree/submodule.");
                return ExitCode::from(1);
            }
        }
    };

    if let Err(e) = fs::create_dir_all(&hooks_dir) {
        eprintln!("Error creating hooks directory: {e}");
        return ExitCode::from(1);
    }

    let hook_path = hooks_dir.join("pre-commit");
    if hook_path.exists() {
        if let Ok(content) = fs::read_to_string(&hook_path) {
            if content.contains("slugrails") {
                println!("Pre-commit hook is already installed in {}", hook_path.display());
                return ExitCode::SUCCESS;
            }
        }
    }

    if let Err(e) = fs::write(&hook_path, HOOK_SCRIPT) {
        eprintln!("Error writing hook file: {e}");
        return ExitCode::from(1);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        let _ = fs::set_permissions(&hook_path, perms);
    }

    println!("Successfully installed pre-commit hook into {}", hook_path.display());
    ExitCode::SUCCESS
}

/// Removes the SlugRails hook if present.
pub fn uninstall_hook(repo_root: &Path) -> ExitCode {
    let hook_path = repo_root.join(".git").join("hooks").join("pre-commit");
    if !hook_path.exists() {
        println!("No pre-commit hook found to uninstall.");
        return ExitCode::SUCCESS;
    }

    match fs::read_to_string(&hook_path) {
        Ok(content) if content.contains("slugrails") => {
            if let Err(e) = fs::remove_file(&hook_path) {
                eprintln!("Error removing hook: {e}");
                return ExitCode::from(1);
            }
            println!("Successfully removed SlugRails pre-commit hook.");
            ExitCode::SUCCESS
        }
        _ => {
            println!("Pre-commit hook does not appear to be installed by SlugRails; leaving intact.");
            ExitCode::SUCCESS
        }
    }
}

fn resolve_gitdir(git_file: &Path) -> Option<PathBuf> {
    let content = fs::read_to_string(git_file).ok()?;
    for line in content.lines() {
        if let Some(path_str) = line.strip_prefix("gitdir:") {
            let trimmed = path_str.trim();
            let resolved = git_file.parent()?.join(trimmed);
            return Some(resolved);
        }
    }
    None
}
