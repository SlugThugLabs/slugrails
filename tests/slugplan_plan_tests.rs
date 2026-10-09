use slugrails::cli::plan::{plan_architecture, run_plan};
use serde_json::Value;
use std::fs;
use std::process::ExitCode;
use tempfile::tempdir;

#[test]
fn test_plan_architecture_default_layered() {
    let dir = tempdir().expect("failed to create temp dir");
    let root = dir.path();

    let output = plan_architecture(root, None, None, None, None).expect("plan_architecture should succeed");
    assert!(output.contains("SlugPlan successfully converged architectural intent"));

    // 1. Verify .planning/ARCHITECTURE.md
    let arch_file = root.join(".planning").join("ARCHITECTURE.md");
    assert!(arch_file.exists(), "ARCHITECTURE.md must exist");
    let arch_content = fs::read_to_string(&arch_file).expect("read ARCHITECTURE.md");
    assert!(arch_content.contains("layers = [\"domain\", \"application\", \"infrastructure\"]"));
    assert!(arch_content.contains("application = [\"domain\"]"));
    assert!(arch_content.contains("infrastructure = [\"application\", \"domain\"]"));
    assert!(arch_content.contains("Define system boundaries and human intent here."));
    assert!(arch_content.contains("\"complexity/deep-nesting\""));
    assert!(arch_content.contains("\"hygiene/swallowed-errors\""));
    assert!(arch_content.contains("\"hygiene/no-secrets\""));

    // 2. Verify .planning/slugplan/intent.json
    let intent_file = root.join(".planning").join("slugplan").join("intent.json");
    assert!(intent_file.exists(), "intent.json must exist");
    let intent_raw = fs::read_to_string(&intent_file).expect("read intent.json");
    let intent_json: Value = serde_json::from_str(&intent_raw).expect("parse intent.json");

    assert_eq!(intent_json["style"], "discovered");
    assert_eq!(intent_json["intent"], "Define system boundaries and human intent here.");
    assert_eq!(intent_json["created_by"], "slugplan");
    let layers = intent_json["layers"]
        .as_array()
        .expect("layers should be array");
    assert_eq!(layers, &["domain", "application", "infrastructure"]);

    // 3. Verify tests/slugrails.rs
    let harness_file = root.join("tests").join("slugrails.rs");
    assert!(harness_file.exists(), "tests/slugrails.rs must exist");
    let harness_content = fs::read_to_string(&harness_file).expect("read slugrails.rs");
    assert!(harness_content.contains("test_architecture_contract_enforced"));
}

#[test]
fn test_plan_architecture_hexagonal_style_and_custom_intent() {
    let dir = tempdir().expect("failed to create temp dir");
    let root = dir.path();

    let custom_intent = "Financial ledger core using hexagonal boundaries";
    let output = plan_architecture(root, Some(custom_intent), Some("hexagonal"), None, None)
        .expect("plan_architecture with hexagonal should succeed");
    assert!(output.contains("SlugPlan successfully converged architectural intent"));

    // 1. Verify ARCHITECTURE.md
    let arch_file = root.join(".planning").join("ARCHITECTURE.md");
    let arch_content = fs::read_to_string(&arch_file).expect("read ARCHITECTURE.md");
    assert!(arch_content.contains("layers = [\"domain\", \"ports\", \"adapters\", \"driver\"]"));
    assert!(arch_content.contains("adapters = [\"ports\", \"domain\"]"));
    assert!(arch_content.contains("driver = [\"adapters\", \"ports\"]"));
    assert!(arch_content.contains(custom_intent));

    // 2. Verify intent.json
    let intent_file = root.join(".planning").join("slugplan").join("intent.json");
    let intent_raw = fs::read_to_string(&intent_file).expect("read intent.json");
    let intent_json: Value = serde_json::from_str(&intent_raw).expect("parse intent.json");
    assert_eq!(intent_json["style"], "hexagonal");
    assert_eq!(intent_json["intent"], custom_intent);
    let layers = intent_json["layers"]
        .as_array()
        .expect("layers should be array");
    assert_eq!(layers, &["domain", "ports", "adapters", "driver"]);

    // 3. Verify tests/slugrails.rs
    assert!(root.join("tests").join("slugrails.rs").exists());
}

#[test]
fn test_plan_architecture_cli_style_with_rust_cargo_toml() {
    let dir = tempdir().expect("failed to create temp dir");
    let root = dir.path();

    // Create Cargo.toml to trigger Rust-specific guards
    let cargo_toml = root.join("Cargo.toml");
    fs::write(
        &cargo_toml,
        "[package]\nname = \"dummy-cli\"\nversion = \"0.1.0\"\n",
    )
    .expect("write Cargo.toml");

    let custom_intent = "Cross-platform terminal runner";
    plan_architecture(root, Some(custom_intent), Some("cli"), None, None)
        .expect("plan_architecture with cli style should succeed");

    // 1. Verify ARCHITECTURE.md contains CLI layers and Rust guards
    let arch_file = root.join(".planning").join("ARCHITECTURE.md");
    let arch_content = fs::read_to_string(&arch_file).expect("read ARCHITECTURE.md");
    assert!(arch_content.contains("layers = [\"domain\", \"engine\", \"cli\"]"));
    assert!(arch_content.contains("cli = [\"engine\", \"domain\"]"));
    assert!(arch_content.contains("\"languages/rust/no-unwrap\""));
    assert!(arch_content.contains("\"languages/rust/unsafe-policy\""));

    // 2. Verify intent.json contains Rust guards
    let intent_file = root.join(".planning").join("slugplan").join("intent.json");
    let intent_raw = fs::read_to_string(&intent_file).expect("read intent.json");
    let intent_json: Value = serde_json::from_str(&intent_raw).expect("parse intent.json");
    assert_eq!(intent_json["style"], "cli");
    let enforce = intent_json["enforced_guards"]
        .as_array()
        .expect("enforced_guards array");
    assert!(enforce.iter().any(|g| g == "languages/rust/unsafe-policy"));
    assert!(enforce.iter().any(|g| g == "languages/rust/no-unwrap"));
}

#[test]
fn test_plan_architecture_custom_agreed_terms_from_chat() {
    let dir = tempdir().expect("failed to create temp dir");
    let root = dir.path();

    // Simulate custom terms agreed between developer and AI in chat
    let agreed_layers = vec!["core_models".to_string(), "query_engine".to_string(), "rest_api".to_string()];
    let agreed_rules = vec!["complexity/source-limits".to_string(), "hygiene/no-secrets".to_string()];

    plan_architecture(
        root,
        Some("Custom e-commerce search service"),
        None,
        Some(agreed_layers),
        Some(agreed_rules),
    )
    .expect("custom agreed plan should succeed");

    let arch_file = root.join(".planning").join("ARCHITECTURE.md");
    let arch_content = fs::read_to_string(&arch_file).expect("read ARCHITECTURE.md");
    assert!(arch_content.contains("layers = [\"core_models\", \"query_engine\", \"rest_api\"]"));
    assert!(arch_content.contains("enforce = [\"complexity/source-limits\", \"hygiene/no-secrets\"]"));
}

#[test]
fn test_run_plan_cli_entrypoint() {
    let dir = tempdir().expect("failed to create temp dir");
    let root = dir.path();

    let args = vec![
        "slugrails".to_string(),
        "plan".to_string(),
        "--project".to_string(),
        root.display().to_string(),
        "--intent".to_string(),
        "Distributed event orchestrator".to_string(),
        "--style".to_string(),
        "hexagonal".to_string(),
    ];

    let exit_code = run_plan(args.into_iter());
    assert_eq!(exit_code, ExitCode::SUCCESS);

    assert!(root.join(".planning").join("ARCHITECTURE.md").exists());
    assert!(root.join(".planning").join("slugplan").join("intent.json").exists());
    assert!(root.join("tests").join("slugrails.rs").exists());
}
