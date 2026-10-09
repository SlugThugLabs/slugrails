use slugrails::analyzer::module_fan_out;

#[test]
fn fan_out_counts_imports_and_imported_by() {
    let files = vec![
        (
            "src/a.rs".to_string(),
            "use crate::b::thing;\nuse crate::c::other;\nuse std::fmt;\n".to_string(),
        ),
        ("src/b.rs".to_string(), "use crate::a::alpha;\n".to_string()),
        ("src/c.rs".to_string(), String::new()),
    ];
    let fan_out = module_fan_out(&files);
    assert_eq!(fan_out["src/a.rs"].imports, 3);
    assert_eq!(fan_out["src/b.rs"].imports, 1);
    assert_eq!(fan_out["src/a.rs"].imported_by, 1);
    assert_eq!(fan_out["src/b.rs"].imported_by, 1);
    assert_eq!(fan_out["src/c.rs"].imported_by, 1);
}

#[test]
fn grouped_and_nested_imports_resolve_to_the_module() {
    let files = vec![
        ("src/analyzer/tokenizer.rs".to_string(), String::new()),
        (
            "src/guards/runner.rs".to_string(),
            "use crate::analyzer::tokenizer::{tokenize_source};\n".to_string(),
        ),
    ];
    let fan_out = module_fan_out(&files);
    assert_eq!(fan_out["src/analyzer/tokenizer.rs"].imported_by, 1);
}

#[test]
fn empty_input_yields_empty_map() {
    assert!(module_fan_out(&[]).is_empty());
}
