//! Git hook installation and management.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HOOK_BLOCK: &str = r#"# >>> slugrails pre-commit gate >>>
if command -v slugrails >/dev/null 2>&1; then
    slugrails check --diff || exit 1
elif command -v slugrails-mcp >/dev/null 2>&1; then
    slugrails-mcp check --diff || exit 1
else
    echo "Error: slugrails not found in PATH. Architectural rails cannot be verified." >&2
    echo "Please install slugrails or commit with --no-verify if explicitly authorized." >&2
    exit 1
fi
# <<< slugrails pre-commit gate <<<
"#;

/// Entry point for `slugrails hook` subcommand.
pub fn run_hook<I: Iterator<Item = String>>(args: I) -> ExitCode {
    let args_vec: Vec<String> = args.collect();
    if crate::cli::args::has_help_flag(&args_vec) {
        println!("Usage: slugrails hook [install|uninstall] [--root <path>]");
        return ExitCode::SUCCESS;
    }
    let root = crate::cli::args::extract_root(&args_vec);
    let mut non_flags = Vec::new();
    let mut skip_next = false;
    for arg in &args_vec {
        if skip_next {
            skip_next = false;
            continue;
        }
        if matches!(arg.as_str(), "--root" | "-r" | "--project" | "-p") {
            skip_next = true;
            continue;
        }
        if !arg.starts_with('-') {
            non_flags.push(arg.as_str());
        }
    }
    let sub = non_flags.first().copied().unwrap_or("install");
    match sub {
        "install" => install_hook(&root),
        "uninstall" => uninstall_hook(&root),
        _ => {
            eprintln!("Usage: slugrails hook [install|uninstall]");
            ExitCode::from(2)
        }
    }
}

/// Resolves the Git hooks directory, supporting normal repositories, worktrees, and submodules.
fn resolve_hooks_dir(repo_root: &Path) -> Option<PathBuf> {
    let git_dir = repo_root.join(".git");
    if git_dir.is_dir() {
        Some(git_dir.join("hooks"))
    } else if git_dir.is_file() {
        resolve_gitdir(&git_dir).map(|d| d.join("hooks"))
    } else {
        None
    }
}

/// Installs the `SlugRails` pre-commit hook into `.git/hooks/pre-commit`.
#[must_use]
pub fn install_hook(repo_root: &Path) -> ExitCode {
    let git_dir = repo_root.join(".git");
    if !git_dir.exists() {
        eprintln!("Error: Not a git repository (missing .git directory).");
        return ExitCode::from(1);
    }

    let Some(hooks_dir) = resolve_hooks_dir(repo_root) else {
        eprintln!("Error: Unable to resolve gitdir for worktree/submodule.");
        return ExitCode::from(1);
    };

    if let Err(e) = fs::create_dir_all(&hooks_dir) {
        eprintln!("Error creating hooks directory: {e}");
        return ExitCode::from(1);
    }

    let hook_path = hooks_dir.join("pre-commit");
    let content = if hook_path.exists() {
        match fs::read_to_string(&hook_path) {
            Ok(c) if c.contains("slugrails") => {
                println!(
                    "Pre-commit hook is already installed in {}",
                    hook_path.display()
                );
                return ExitCode::SUCCESS;
            }
            Ok(c) => format!("{c}\n\n{HOOK_BLOCK}"),
            Err(e) => {
                eprintln!("Error reading existing hook: {e}");
                return ExitCode::from(1);
            }
        }
    } else {
        format!("#!/usr/bin/env sh\n\n{HOOK_BLOCK}")
    };

    if let Err(e) = fs::write(&hook_path, content) {
        eprintln!("Error writing hook file: {e}");
        return ExitCode::from(1);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        let _ = fs::set_permissions(&hook_path, perms);
    }

    println!(
        "Successfully installed pre-commit hook into {}",
        hook_path.display()
    );
    ExitCode::SUCCESS
}

/// Removes the `SlugRails` hook if present.
#[must_use]
pub fn uninstall_hook(repo_root: &Path) -> ExitCode {
    let Some(hooks_dir) = resolve_hooks_dir(repo_root) else {
        println!("No pre-commit hook found to uninstall.");
        return ExitCode::SUCCESS;
    };
    let hook_path = hooks_dir.join("pre-commit");
    if !hook_path.exists() {
        println!("No pre-commit hook found to uninstall.");
        return ExitCode::SUCCESS;
    }

    match fs::read_to_string(&hook_path) {
        Ok(content) if content.contains("slugrails") => {
            let cleaned = strip_slugrails_hook(&content);
            if cleaned.trim().is_empty() || cleaned.trim() == "#!/usr/bin/env sh" {
                if let Err(e) = fs::remove_file(&hook_path) {
                    eprintln!("Error removing hook: {e}");
                    return ExitCode::from(1);
                }
            } else if let Err(e) = fs::write(&hook_path, cleaned) {
                eprintln!("Error updating hook file: {e}");
                return ExitCode::from(1);
            }
            println!("Successfully removed SlugRails pre-commit hook.");
            ExitCode::SUCCESS
        }
        _ => {
            println!(
                "Pre-commit hook does not appear to be installed by SlugRails; leaving intact."
            );
            ExitCode::SUCCESS
        }
    }
}

fn strip_slugrails_hook(content: &str) -> String {
    let mut out = Vec::new();
    let mut skipping = false;
    for line in content.lines() {
        if line.contains(">>> slugrails pre-commit gate >>>") {
            skipping = true;
            continue;
        }
        if line.contains("<<< slugrails pre-commit gate <<<") {
            skipping = false;
            continue;
        }
        if !skipping && !line.contains("slugrails") {
            out.push(line);
        }
    }
    out.join("\n")
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
