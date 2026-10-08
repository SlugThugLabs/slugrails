//! Execution steps for the interactive setup menu options.

use crate::error::MenuError;
use crate::util::{get_slugthug_home, get_tests_dir};
use std::io::{BufRead as _, Write as _};
use std::path::PathBuf;

pub fn running_binary() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("codeguards-mcp"))
}

pub fn install_step() {
    println!();
    let source = running_binary();
    let bin_dir = get_slugthug_home().join("bin");
    let target = bin_dir.join(crate::util::get_binary_name());

    if let Err(e) = std::fs::create_dir_all(&bin_dir) {
        eprintln!("Failed to create {}: {e}", bin_dir.display());
        return;
    }

    match std::fs::copy(&source, &target) {
        Ok(_) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(mut perms) = std::fs::metadata(&target).map(|m| m.permissions()) {
                    perms.set_mode(0o755);
                    let _ = std::fs::set_permissions(&target, perms);
                }
            }
            println!("Installed CodeGuards to {}", target.display());
            println!("Ensure {} is in your $PATH.", bin_dir.display());
        }
        Err(e) => eprintln!("Failed to copy binary to {}: {e}", target.display()),
    }
}

pub fn connect_step<F>(mut read_choice_fn: F)
where
    F: FnMut() -> Result<Option<usize>, MenuError>,
{
    println!();
    println!("Supported AI agent CLIs:");
    println!("  1) Hermes Agent (`hermes mcp add ...`)");
    println!("  2) Claude Code   (`claude mcp add ...`)");
    println!("  3) Grok          (`grok mcp add ...`)");
    println!("  4) Codex         (`codex mcp add ...`)");
    print!("\nChoose an agent [1-4]: ");
    std::io::stdout().flush().ok();

    let choice = read_choice_fn().ok().flatten().unwrap_or(0);
    let (name, cli) = match choice {
        1 => ("Hermes", "hermes"),
        2 => ("Claude Code", "claude"),
        3 => ("Grok", "grok"),
        4 => ("Codex", "codex"),
        _ => {
            println!("Invalid choice.");
            return;
        }
    };

    let bin_path = get_slugthug_home().join("bin").join(crate::util::get_binary_name());
    let exec_path = if bin_path.exists() {
        bin_path
    } else {
        running_binary()
    };

    // Pre-flight check: is the CLI on PATH?
    if which::which(cli).is_err() {
        println!("\n'{cli}' CLI was not found on your system PATH.");
        println!("Please install {name} first, or connect manually using:");
        println!("  {cli} mcp add codeguards -- {} serve\n", exec_path.display());
        return;
    }

    println!("\nConnecting CodeGuards to {name}...");

    // Remove existing with 5s timeout
    let mut remove_cmd = std::process::Command::new(cli);
    remove_cmd
        .args(["mcp", "remove", "codeguards"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    let _ = run_with_timeout(&mut remove_cmd, std::time::Duration::from_secs(5));

    // Add server with 5s timeout
    let mut add_cmd = std::process::Command::new(cli);
    add_cmd
        .args(["mcp", "add", "codeguards", "--"])
        .arg(&exec_path)
        .arg("serve");
    let status = run_with_timeout(&mut add_cmd, std::time::Duration::from_secs(5));

    match status {
        Some(s) if s.success() => {
            println!("Connected successfully! Verify with: {cli} mcp list");
        }
        _ => {
            println!("Could not connect automatically.");
            println!("Manual command: {cli} mcp add codeguards -- {} serve", exec_path.display());
        }
    }
}

/// Spawns a child process and waits up to `timeout`, killing it cleanly if it stalls.
fn run_with_timeout(
    cmd: &mut std::process::Command,
    timeout: std::time::Duration,
) -> Option<std::process::ExitStatus> {
    let mut child = cmd.spawn().ok()?;
    let start = std::time::Instant::now();
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            return Some(status);
        }
        if start.elapsed() >= timeout {
            let _ = child.kill();
            return None;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

pub fn other_client_step() {
    println!();
    let bin_path = get_slugthug_home().join("bin").join(crate::util::get_binary_name());
    let exec_path = if bin_path.exists() {
        bin_path
    } else {
        running_binary()
    };

    println!("CodeGuards is a standard stdio MCP server:");
    println!("  Server name:  codeguards");
    println!("  Command:      {}", exec_path.display());
    println!("  Arguments:    serve\n");
    println!("For clients reading .mcp.json (OpenCode, Claude, Cursor):");
    println!(
        "{{\n  \"mcpServers\": {{\n    \"codeguards\": {{\n      \"command\": \"{}\",\n      \"args\": [\"serve\"]\n    }}\n  }}\n}}",
        exec_path.display()
    );
}

pub fn guard_catalog_step() {
    println!();
    match crate::library::ensure_test_library_seeded() {
        Ok(catalog) => {
            println!(
                "Guard-Test Library Catalog ({} tests available in {}):",
                catalog.total_tests,
                get_tests_dir().display()
            );
            for (id, entry) in &catalog.tests {
                println!("  - {:<30} [{}]", id, entry.category);
                println!("    Summary: {}", entry.summary);
                if !entry.aliases.is_empty() {
                    println!("    Aliases: {}", entry.aliases.join(", "));
                }
            }
        }
        Err(e) => eprintln!("Failed to load guard-test library: {e}"),
    }
}

pub fn confirm_serve() -> Result<bool, MenuError> {
    println!();
    println!("The MCP server (`serve`) is normally started automatically by your AI agent.");
    print!("Start the server in this terminal now? [y/N]: ");
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line)?;
    Ok(matches!(
        line.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}
