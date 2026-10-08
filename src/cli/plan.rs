//! SlugPlan: Intent Convergence Engine & Architectural Advisor.
//!
//! Conducts architectural interviews, discovers domain terms and layers,
//! generates ARCHITECTURE.md contracts, and scaffolds tests/slugrails.rs.

use crate::cli::args::{extract_option, extract_root, has_help_flag};
use crate::cli::scaffold_tests::scaffold_tests_harness;
use crate::contract::discovery::inspect_project;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

pub fn run_plan(args: impl Iterator<Item = String>) -> ExitCode {
    let args_vec: Vec<String> = args.collect();
    if has_help_flag(&args_vec) {
        println!(
            "Usage: slugrails plan [--root <path>] [--intent <desc>] [--style <layered|hexagonal|cli>] [--layers <l1,l2,...>]\n\n\
             Discovers architectural domain terms and auto-scaffolds contracts + test harness."
        );
        return ExitCode::SUCCESS;
    }
    let root = extract_root(&args_vec);
    let intent_input = extract_option(&args_vec, "--intent", Some("-i"));
    let style_input = extract_option(&args_vec, "--style", Some("-s"));
    let layers_input = extract_option(&args_vec, "--layers", Some("-l"))
        .map(|s| s.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect());

    match plan_architecture(
        &root,
        intent_input.as_deref(),
        style_input.as_deref(),
        layers_input,
        None,
    ) {
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
    custom_layers: Option<Vec<String>>,
    custom_rules: Option<Vec<String>>,
) -> std::io::Result<String> {
    let plan_dir = root.join(".planning");
    fs::create_dir_all(&plan_dir)?;

    let intent = intent_desc.unwrap_or("Define system boundaries and human intent here.");
    let inspection = inspect_project(root, intent, style_hint);

    let layers = custom_layers.unwrap_or(inspection.candidate_layers);
    let enforce = custom_rules.unwrap_or(inspection.recommended_rules);

    let layers_str = layers.iter().map(|l| format!("\"{l}\"")).collect::<Vec<_>>().join(", ");
    let enforce_str = enforce.iter().map(|e| format!("\"{e}\"")).collect::<Vec<_>>().join(", ");

    let mut deps_str = String::new();
    for (k, v) in &inspection.allowed_dependencies {
        let targets = v.iter().map(|t| format!("\"{t}\"")).collect::<Vec<_>>().join(", ");
        deps_str.push_str(&format!("{k} = [{targets}]\n"));
    }

    let arch_path = plan_dir.join("ARCHITECTURE.md");
    let arch_content = format!(
        "+++\n\
         layers = [{layers_str}]\n\
         enforce = [{enforce_str}]\n\n\
         [allowed_dependencies]\n\
         {deps_str}\
         +++\n\n\
         # Intent & Architectural Contract\n\n\
         {intent}\n"
    );
    fs::write(&arch_path, arch_content)?;

    let slugplan_dir = crate::util::get_project_plan_dir(root);
    fs::create_dir_all(&slugplan_dir)?;
    let intent_json_path = slugplan_dir.join("intent.json");
    let intent_data = serde_json::json!({
        "intent": intent,
        "style": style_hint.unwrap_or("discovered"),
        "detected_language": inspection.detected_language,
        "discovered_modules": inspection.existing_modules,
        "layers": layers,
        "enforced_guards": enforce,
        "created_by": "slugplan"
    });
    fs::write(&intent_json_path, serde_json::to_string_pretty(&intent_data)?)?;

    let harness_path = scaffold_tests_harness(root)?;
    let _ = crate::library::ensure_test_library_seeded();

    Ok(format!(
        "SlugPlan successfully converged architectural intent:\n\
         - Language: {}\n\
         - Modules Found: {:?}\n\
         - Contract: {}\n\
         - Test Harness: {}\n\
         Run 'cargo test --test slugrails' or 'slugrails check' to enforce intent.",
        inspection.detected_language,
        inspection.existing_modules,
        arch_path.display(),
        harness_path.display()
    ))
}
