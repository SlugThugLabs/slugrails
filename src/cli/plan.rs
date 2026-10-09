//! `SlugPlan`: Intent Convergence Engine & Architectural Advisor.
//!
//! Conducts architectural interviews, discovers domain terms and layers,
//! generates ARCHITECTURE.md contracts, and scaffolds tests/slugrails.rs.

use crate::cli::args::{extract_option, extract_root, has_help_flag};
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
    let layers_input = extract_option(&args_vec, "--layers", Some("-l")).map(|s| {
        s.split(',')
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect()
    });

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

/// Converges and plans architecture contract.
///
/// # Errors
///
/// Returns an I/O error if writing the contract or baseline fails.
pub fn plan_architecture(
    root: &Path,
    intent_desc: Option<&str>,
    style_hint: Option<&str>,
    custom_layers: Option<Vec<String>>,
    custom_rules: Option<Vec<String>>,
) -> std::io::Result<String> {
    crate::contract::plan_architecture(root, intent_desc, style_hint, custom_layers, custom_rules)
}
