#![allow(warnings, clippy::all, clippy::pedantic)]

use slugrails::library::GuardCatalog;
use slugrails::types::GuardTestDefinition;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn def(id: &str, name: &str, tags: &[&str], aliases: &[&str]) -> GuardTestDefinition {
    GuardTestDefinition {
        id: id.to_string(),
        name: name.to_string(),
        category: "structural".to_string(),
        version: "1.0.0".to_string(),
        summary: "test guard".to_string(),
        tags: tags.iter().map(ToString::to_string).collect(),
        aliases: aliases.iter().map(ToString::to_string).collect(),
        engine: "builtin".to_string(),
        default_params: BTreeMap::new(),
        remediation: "fix it".to_string(),
    }
}

fn catalog(defs: &[GuardTestDefinition]) -> GuardCatalog {
    let defs: Vec<(GuardTestDefinition, PathBuf)> = defs
        .iter()
        .map(|d| (d.clone(), PathBuf::from("/x")))
        .collect();
    GuardCatalog::from_definitions(&defs)
}

#[test]
fn resolve_by_id_name_and_alias_case_insensitive() {
    let defs = vec![def(
        "structural/layer-deps",
        "layer-deps",
        &["layers"],
        &["layers", "deps"],
    )];
    let cat = catalog(&defs);
    assert!(cat.resolve("structural/layer-deps").is_some());
    assert!(
        cat.resolve("LAYER-DEPS").is_some(),
        "name lookup must be case-insensitive"
    );
    assert!(
        cat.resolve("DEPS").is_some(),
        "alias lookup must be case-insensitive"
    );
    assert_eq!(cat.resolve("deps").unwrap().id, "structural/layer-deps");
}

#[test]
fn resolve_unknown_query_returns_none() {
    let defs = vec![def("structural/x", "x", &[], &[])];
    let cat = catalog(&defs);
    assert!(cat.resolve("nonexistent").is_none());
}

#[test]
fn duplicate_exact_name_match_detected() {
    let defs = vec![def(
        "structural/layers",
        "layers",
        &["layers"],
        &["layer-deps"],
    )];
    let cat = catalog(&defs);
    assert!(
        cat.find_potential_duplicate("layers", &[]).is_some(),
        "exact name match"
    );
    assert!(
        cat.find_potential_duplicate("LAYER-DEPS", &[]).is_some(),
        "alias match must be detected"
    );
}

#[test]
fn duplicate_tag_overlap_detected_at_three_shared_tags() {
    let defs = vec![def(
        "structural/layers",
        "layers",
        &["layers", "imports", "deps"],
        &[],
    )];
    let cat = catalog(&defs);
    // Three shared tags -> duplicate.
    let tags = vec![
        "LAYERS".to_string(),
        "imports".to_string(),
        "deps".to_string(),
    ];
    assert!(cat.find_potential_duplicate("fresh-name", &tags).is_some());
    // Two shared tags -> not a duplicate.
    let tags = vec!["LAYERS".to_string(), "imports".to_string()];
    assert!(cat.find_potential_duplicate("fresh-name", &tags).is_none());
}

#[test]
fn duplicate_none_when_distinct() {
    let defs = vec![def(
        "structural/layers",
        "layers",
        &["layers", "imports"],
        &[],
    )];
    let cat = catalog(&defs);
    let tags = vec![
        "secrets".to_string(),
        "crypto".to_string(),
        "tokens".to_string(),
    ];
    assert!(
        cat.find_potential_duplicate("secret-scanner", &tags)
            .is_none()
    );
}
