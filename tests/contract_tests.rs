#![allow(warnings, clippy::all, clippy::pedantic)]

use slugrails::contract::{
    ArchitectureContract, load_architecture, parse_frontmatter, validate_architecture,
    validate_contract,
};
use slugrails::error::{ContractError, SlugRailsError};
use slugrails::library::catalog::GuardCatalog;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_parse_frontmatter_valid() {
    let content = r#"+++
modules = ["server", "storage"]
layers = ["transport", "storage"]
enforce = ["no_unwrap", "source_limits"]

[allowed_dependencies]
server = ["storage"]
storage = []
+++

# Architecture Title
Some body text.
"#;

    let (contract, body) = parse_frontmatter(content).unwrap();
    assert_eq!(contract.modules, vec!["server", "storage"]);
    assert_eq!(contract.layers, vec!["transport", "storage"]);
    assert_eq!(contract.enforce, vec!["no_unwrap", "source_limits"]);
    assert_eq!(
        contract.allowed_dependencies.get("server").unwrap(),
        &vec!["storage"]
    );
    assert!(body.contains("Some body text."));
}

#[test]
fn frontmatter_without_fences_returns_default_contract() {
    let content = "# Just a heading\nNo frontmatter at all.\n";
    let (contract, body) = parse_frontmatter(content).expect("no fences = default");
    assert!(contract.modules.is_empty());
    assert!(contract.enforce.is_empty());
    assert_eq!(body, content);
}

#[test]
fn frontmatter_unclosed_fence_is_an_error() {
    let content = "+++\nmodules = [\"x\"]\n# never closed\n";
    let err = parse_frontmatter(content).expect_err("unclosed fence must fail");
    assert!(matches!(
        err,
        SlugRailsError::Contract(ContractError::UnclosedFrontmatter)
    ));
}

#[test]
fn frontmatter_malformed_toml_is_an_error() {
    let content = "+++\nmodules = [\"unclosed\n+++\nbody\n";
    let err = parse_frontmatter(content).expect_err("malformed TOML must fail");
    assert!(matches!(err, SlugRailsError::TomlParse { .. }));
}

#[test]
fn frontmatter_wrong_field_types_is_an_error() {
    let content = "+++\nmodules = \"not-a-list\"\n+++\n";
    assert!(parse_frontmatter(content).is_err());
}

#[test]
fn load_architecture_missing_file_errors() {
    let dir = tempdir().expect("tempdir");
    let err = load_architecture(dir.path()).expect_err("no .planning dir");
    assert!(matches!(
        err,
        SlugRailsError::Contract(ContractError::MissingContract { .. })
    ));
}

#[test]
fn test_validate_architecture_with_missing_guards() {
    let dir = tempdir().unwrap();
    let plan_dir = dir.path().join(".planning");
    fs::create_dir_all(&plan_dir).unwrap();

    let arch_content = r#"+++
modules = ["server"]
enforce = ["custom_payment_rule"]
+++
"#;
    fs::write(plan_dir.join("ARCHITECTURE.md"), arch_content).unwrap();

    let catalog = GuardCatalog::default();
    let result = validate_architecture(dir.path(), &catalog).unwrap();

    assert!(!result.is_valid);
    assert_eq!(result.missing_guards, vec!["custom_payment_rule"]);
    assert_eq!(result.errors.len(), 1);
}

#[test]
fn validate_declared_module_warning_when_absent() {
    let dir = tempdir().expect("tempdir");
    fs::create_dir_all(dir.path().join(".planning")).unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(
        dir.path().join(".planning/ARCHITECTURE.md"),
        "+++\nmodules = [\"does_not_exist_on_disk\"]\n+++\n",
    )
    .unwrap();
    let catalog = GuardCatalog::default();
    let result = validate_architecture(dir.path(), &catalog).expect("validation runs");
    // Missing modules are warnings, not errors — still valid.
    assert!(result.is_valid);
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.contains("does_not_exist_on_disk")),
        "expected missing-module warning, got {:?}",
        result.warnings
    );
}

#[test]
fn test_validate_contract_pure_in_memory() {
    let mut contract = ArchitectureContract::default();
    contract.modules = vec!["core".to_string(), "ghost".to_string()];
    contract.enforce = vec![
        "complexity/source-limits".to_string(),
        "unknown/rule".to_string(),
    ];

    let catalog = GuardCatalog::default();
    let existing_modules = vec!["core".to_string()];

    let result = validate_contract(&contract, &existing_modules, &catalog);
    assert!(!result.is_valid);
    assert_eq!(result.missing_guards, vec!["unknown/rule"]);
    assert_eq!(result.ready_guards, vec!["complexity/source-limits"]);
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.warnings.len(), 1);
    assert!(result.warnings[0].contains("ghost"));
}
