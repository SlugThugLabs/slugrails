#![allow(warnings, clippy::all, clippy::pedantic)]

use slugrails::analyzer::{function_spans, has_doc_comment};

#[test]
fn rust_spans_carry_bounds_params_visibility_and_docs() {
    let source = "\
/// Adds one.
pub fn add(a: u8, b: u8) -> u8 { a + b }

pub async fn go<T, U>(x: T, y: Vec<U>, z: HashMap<K, V>) {}

fn private() {}
";
    let spans = function_spans(source);
    assert_eq!(spans.len(), 3, "got {spans:?}");

    assert_eq!(spans[0].name, "add");
    assert_eq!(spans[0].start_line, 2);
    assert_eq!(spans[0].end_line, 2);
    assert_eq!(spans[0].param_count, 2);
    assert!(spans[0].is_pub);
    assert!(spans[0].has_doc, "leading /// must count as documentation");

    assert_eq!(spans[1].name, "go");
    assert_eq!(spans[1].param_count, 3);
    assert!(spans[1].is_pub);
    assert!(!spans[1].has_doc);

    assert_eq!(spans[2].name, "private");
    assert_eq!(spans[2].param_count, 0);
    assert!(!spans[2].is_pub);
}

#[test]
fn generic_bound_parens_are_not_the_parameter_list() {
    let source = "pub fn run<T: Fn(u8) -> u8>(callback: T, count: usize) {}\n";
    let spans = function_spans(source);
    assert_eq!(spans.len(), 1, "got {spans:?}");
    assert_eq!(spans[0].param_count, 2);
}

#[test]
fn multi_line_signature_span_covers_body() {
    let source = "pub fn multi(\n    a: u8,\n    b: u8,\n) -> u8 {\n    a + b\n}\n";
    let spans = function_spans(source);
    assert_eq!(spans.len(), 1, "got {spans:?}");
    assert_eq!(spans[0].param_count, 2);
    assert_eq!(spans[0].start_line, 1);
    assert_eq!(spans[0].end_line, 6);
}

#[test]
fn trait_declaration_ends_at_semicolon() {
    let source = "pub trait T {\n    fn method(&self);\n}\n";
    let spans = function_spans(source);
    let method = spans
        .iter()
        .find(|span| span.name == "method")
        .expect("method span");
    assert_eq!(method.start_line, 2);
    assert_eq!(method.end_line, 2);
    assert_eq!(method.param_count, 1, "&self counts as a parameter");
}

#[test]
fn python_def_span_uses_indentation_and_detects_docstring() {
    let source = "\
def outer(a, b):
    if a:
        return b
    return a
";
    let spans = function_spans(source);
    assert_eq!(spans.len(), 1, "got {spans:?}");
    assert_eq!(spans[0].name, "outer");
    assert_eq!(spans[0].start_line, 1);
    assert_eq!(spans[0].end_line, 4);
    assert_eq!(spans[0].param_count, 2);
    assert!(spans[0].is_pub);
    assert!(!spans[0].has_doc);

    let documented = "def documented():\n    \"\"\"Doc.\"\"\"\n    return 1\n";
    let spans = function_spans(documented);
    assert!(spans[0].has_doc, "docstring must count as documentation");
}

#[test]
fn doc_comment_is_seen_through_attributes() {
    let source = "/// Doc.\n#[inline]\npub fn f() {}\n";
    assert!(has_doc_comment(source, 3));
}

#[test]
fn doc_comment_search_stops_at_other_code() {
    let source = "let x = 1;\npub fn f() {}\n";
    assert!(!has_doc_comment(source, 2));
    assert!(!has_doc_comment("pub fn f() {}\n", 1));
    assert!(!has_doc_comment(source, 99));
}
