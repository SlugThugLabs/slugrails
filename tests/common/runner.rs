//! Opaque-box process runner for SlugRails.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Result of executing a command via the opaque-box test runner.
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Process exit code.
    pub exit_code: i32,
    /// Captured standard output.
    pub stdout: String,
    /// Captured standard error.
    pub stderr: String,
}

impl CommandResult {
    /// Returns true if exit code is 0.
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }
}

/// Locates the compiled `slugrails` binary or fallback executable.
pub fn slugrails_bin() -> PathBuf {
    if let Ok(var) = std::env::var("SLUGRAILS_BIN") {
        let target_path = PathBuf::from(var);
        if target_path.exists() {
            return target_path;
        }
    }
    if let Ok(var) = std::env::var("CARGO_BIN_EXE_slugrails") {
        let target_path = PathBuf::from(var);
        if target_path.exists() {
            return target_path;
        }
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        manifest_dir.join("target/debug/slugrails.exe"),
        manifest_dir.join("target/debug/slugrails"),
        manifest_dir.join("target/release/slugrails.exe"),
        manifest_dir.join("target/release/slugrails"),
        PathBuf::from("C:\\Users\\Sly\\CodeGuards-mcp\\target\\debug\\slugrails.exe"),
        PathBuf::from("slugrails.exe"),
        PathBuf::from("slugrails"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return PathBuf::from(candidate);
        }
    }

    manifest_dir.join("target/debug/slugrails.exe")
}

/// Invokes `slugrails` with arguments.
pub fn run_slugrails(args: &[&str], cwd: &Path) -> CommandResult {
    run_slugrails_with_stdin(args, cwd, "")
}

/// Invokes `slugrails` with arguments and standard input data.
pub fn run_slugrails_with_stdin(args: &[&str], cwd: &Path, input: &str) -> CommandResult {
    let bin = slugrails_bin();
    let mut cmd = if bin.exists() {
        Command::new(&bin)
    } else {
        let mut fallback = Command::new("cargo");
        fallback.args(["run", "--quiet", "--bin", "slugrails", "--"]);
        fallback
    };

    cmd.args(args);
    cmd.current_dir(cwd);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(err) => {
            return CommandResult {
                exit_code: -1,
                stdout: String::new(),
                stderr: format!("Spawn failed: {err}"),
            };
        }
    };

    if !input.is_empty() {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(input.as_bytes());
        }
    }

    let output = match child.wait_with_output() {
        Ok(out) => out,
        Err(err) => {
            return CommandResult {
                exit_code: -1,
                stdout: String::new(),
                stderr: format!("Wait failed: {err}"),
            };
        }
    };

    let result = CommandResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    };
    if false {
        let _ = result.is_success();
        let _ = result.stdout.len() + result.stderr.len();
    }
    result
}
