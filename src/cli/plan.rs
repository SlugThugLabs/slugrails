//! SlugPlan: Intent Convergence Engine & Architectural Advisor.
//!
//! Conducts architectural interviews, discovers domain terms and layers,
//! generates ARCHITECTURE.md contracts, and scaffolds tests/slugrails.rs.

use crate::cli::args::{extract_option, extract_root};
use crate::cli::scaffold_tests::scaffold_tests_harness;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

pub fn run_plan(args: impl Iterator<Item = String>) -> ExitCode {
    let args_vec: Vec<String> = args.collect();
    let root = extract_root(&args_vec);
    let intent_input = extract_option(&args_vec, "--intent", Some("-i"));
    let style_input = extract_option(&args_vec, "--style", Some("-s"));

    match plan_architecture(&root, intent_input.as_deref(), style_input.as_deref()) {
        Ok(msg) => {
            println!("{msg}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("SlugPlan error: {e}");
            ExitCode::from(1)
        }
    }
}

pub fn plan_architecture(
    root: &Path,
    intent_desc: Option<&str>,
    style_hint: Option<&str>,
) -> std::io::Result<String> {
    let plan_dir = root.join(".planning");
    fs::create_dir_all(&plan_dir)?;

    let (layers, deps) = match style_hint.unwrap_or("layered").to_lowercase().as_str() {
        "hexagonal" | "ports-and-adapters" => (
            vec!["domain", "ports", "adapters", "driver"],
            "adapters = [\"ports\", \"domain\"]\ndriver = [\"adapters\", \"ports\"]",
        ),
        "cli" | "tool" => (
            vec!["domain", "engine", "cli"],
            "cli = [\"engine\", \"domain\"]\nengine = [\"domain\"]",
        ),
        _ => (
            vec!["domain", "application", "infrastructure"],
            "application = [\"domain\"]\ninfrastructure = [\"application\", \"domain\"]",
        ),
    };

    let is_rust = root.join("Cargo.toml").exists();
    let mut enforce = vec![
        "complexity/source-limits",
        "complexity/function-limits",
        "complexity/deep-nesting",
        "quality/no-stubs",
        "hygiene/swallowed-errors",
        "hygiene/no-secrets",
        "hygiene/no-debug-prints",
    ];
    if is_rust {
        enforce.push("languages/rust/no-unwrap");
        enforce.push("languages/rust/unsafe-policy");
    }

    let layers_str = layers
        .iter()
        .map(|l| format!("\"{l}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let enforce_str = enforce
        .iter()
        .map(|e| format!("\"{e}\""))
        .collect::<Vec<_>>()
        .join(", ");

    let arch_path = plan_dir.join("ARCHITECTURE.md");
    let arch_content = format!(
        "+++\n\
         layers = [{layers_str}]\n\
         enforce = [{enforce_str}]\n\n\
         [allowed_dependencies]\n\
         {deps}\n\
         +++\n\n\
         # Intent & Architectural Contract\n\n\
         {}\n",
        intent_desc.unwrap_or("Define system boundaries and human intent here.")
    );
    fs::write(&arch_path, arch_content)?;

    // Write .planning/slugplan/intent.json
    let slugplan_dir = crate::util::get_project_plan_dir(root);
    fs::create_dir_all(&slugplan_dir)?;
    let intent_json_path = slugplan_dir.join("intent.json");
    let intent_data = serde_json::json!({
        "intent": intent_desc.unwrap_or("General production codebase"),
        "style": style_hint.unwrap_or("layered"),
        "layers": layers,
        "enforced_guards": enforce,
        "created_by": "slugplan"
    });
    fs::write(
        &intent_json_path,
        serde_json::to_string_pretty(&intent_data)?,
    )?;

    // Auto-scaffold tests/slugrails.rs
    let harness_path = scaffold_tests_harness(root)?;

    // Ensure global accumulator stack is seeded
    let _ = crate::library::ensure_test_library_seeded();

    Ok(format!(
        "SlugPlan successfully converged architectural intent:\n\
         - Contract: {}\n\
         - Plan/Intent: {}\n\
         - Test Harness: {}\n\
         Run 'cargo test --test slugrails' or 'slugrails check' to enforce intent.",
        arch_path.display(),
        intent_json_path.display(),
        harness_path.display()
    ))
}
