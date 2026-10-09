//! Scaffolds a starter .planning/ARCHITECTURE.md contract for a project.

use crate::error::{Result, SlugRailsError};
use std::fs;
use std::path::Path;

/// Initializes a `SlugRails` architecture contract for the given project root.
///
/// # Errors
///
/// Returns an error if the contract file cannot be written.
pub fn run_init<I: Iterator<Item = String>>(args: I) -> std::process::ExitCode {
    let args_vec: Vec<String> = args.collect();
    if super::args::has_help_flag(&args_vec) {
        println!(
            "Usage: slugrails init [--root <path>]\n\nScaffolds starter .planning/ARCHITECTURE.md contract with auto-discovered modules."
        );
        return std::process::ExitCode::SUCCESS;
    }
    let root_path = super::args::extract_root(&args_vec);
    match initialize_project(&root_path) {
        Ok(msg) => {
            println!("{msg}");
            std::process::ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Initialization error: {e}");
            std::process::ExitCode::from(1)
        }
    }
}

/// Scaffolds an architecture contract in `.planning/ARCHITECTURE.md`.
///
/// # Errors
///
/// Returns [`SlugRailsError::Io`] if creating the planning directory or writing the file fails.
pub fn initialize_project(project_root: &Path) -> Result<String> {
    let plan_dir = project_root.join(".planning");
    let arch_file = plan_dir.join("ARCHITECTURE.md");

    if arch_file.exists() {
        return Ok(format!(
            "Architecture contract already exists at {}",
            arch_file.display()
        ));
    }

    fs::create_dir_all(&plan_dir).map_err(|e| SlugRailsError::Io {
        path: plan_dir.clone(),
        source: e,
    })?;

    let modules = discover_modules(project_root);
    let is_rust = project_root.join("Cargo.toml").exists();

    let mut enforce_rules = vec![
        "\"complexity/source-limits\"".to_string(),
        "\"quality/no-stubs\"".to_string(),
        "\"hygiene/no-secrets\"".to_string(),
        "\"hygiene/no-debug-prints\"".to_string(),
    ];
    if is_rust {
        enforce_rules.push("\"languages/rust/no-unwrap\"".to_string());
    }

    let mod_strings = modules
        .iter()
        .map(|m| format!("\"{m}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let enforce_strings = enforce_rules.join(", ");

    let content = format!(
        "+++\n\
         modules = [{mod_strings}]\n\
         layers = [\"domain\", \"application\", \"infrastructure\"]\n\
         enforce = [{enforce_strings}]\n\n\
         [allowed_dependencies]\n\
         # Declare allowed module dependency flow here\n\
         +++\n\n\
         # Architecture Contract\n\n\
         Define your architecture, layers, and boundaries here.\n\
         SlugRails will continuously verify that code matches this contract.\n"
    );

    fs::write(&arch_file, content).map_err(|e| SlugRailsError::Io {
        path: arch_file.clone(),
        source: e,
    })?;

    // Create .planning/slugrails/ and .planning/slugplan/ directories
    let rails_dir = crate::util::get_project_storage_dir(project_root);
    let _ = fs::create_dir_all(&rails_dir);

    let plan_dir_sub = crate::util::get_project_plan_dir(project_root);
    let _ = fs::create_dir_all(&plan_dir_sub);
    let intent_file = plan_dir_sub.join("intent.json");
    if !intent_file.exists() {
        let default_intent = serde_json::json!({
            "status": "initialized",
            "modules": modules,
            "managed_by": "slugrails"
        });
        let _ = fs::write(
            &intent_file,
            serde_json::to_string_pretty(&default_intent).unwrap_or_default(),
        );
    }

    // Seed test library in ~/.slugthuglabs/slugrails/tests/
    let _ = crate::library::ensure_test_library_seeded();

    Ok(format!(
        "Initialized architecture contract at {}.\n\
         Scaffolded .planning/slugrails/ and .planning/slugplan/\n\
         Run 'slugrails check' to verify your codebase against it.",
        arch_file.display()
    ))
}

fn discover_modules(root: &Path) -> Vec<String> {
    let src_dir = root.join("src");
    let target = if src_dir.is_dir() { &src_dir } else { root };
    let mut modules = Vec::new();

    if let Ok(entries) = fs::read_dir(target) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if name.is_empty() || name == "lib" || name == "main" || name.starts_with('.') {
                continue;
            }
            if path.is_dir()
                || path
                    .extension()
                    .is_some_and(|ext| ext == "rs" || ext == "py" || ext == "ts")
            {
                modules.push(name.to_string());
            }
        }
    }
    modules.sort();
    modules
}
