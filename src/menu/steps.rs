//! Execution steps for the interactive setup menu options.

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
    F: FnMut() -> Result<Option<usize>, Box<dyn std::error::Error>>,
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

    println!("\nConnecting CodeGuards to {name}...");

    // Remove existing
    let _ = std::process::Command::new(cli)
        .args(["mcp", "remove", "codeguards"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    // Add server
    let status = std::process::Command::new(cli)
        .args(["mcp", "add", "codeguards", "--"])
        .arg(&exec_path)
        .arg("serve")
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("Connected successfully! Verify with: {cli} mcp list");
        }
        _ => {
            println!(
                "Could not invoke '{cli}' CLI. Make sure '{cli}' is installed and on your PATH."
            );
            println!(
                "Manual command: {cli} mcp add codeguards -- {} serve",
                exec_path.display()
            );
        }
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

pub fn confirm_serve() -> Result<bool, Box<dyn std::error::Error>> {
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
