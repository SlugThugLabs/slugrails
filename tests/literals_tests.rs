use codeguards_mcp::analyzer::{LiteralKind, extract_literals};

#[test]
fn numeric_and_string_literals_carry_line_and_context() {
    let source = "let code = 8080;\nlet url = \"https://example.com\";\n";
    let literals = extract_literals(source);
    assert_eq!(literals.len(), 2, "got {literals:?}");
    assert_eq!(literals[0].kind, LiteralKind::Numeric);
    assert_eq!(literals[0].value, "8080");
    assert_eq!(literals[0].line, 1);
    assert!(literals[0].context.contains("8080"));
    assert_eq!(literals[1].kind, LiteralKind::String);
    assert_eq!(literals[1].value, "https://example.com");
    assert_eq!(literals[1].line, 2);
}

#[test]
fn digits_inside_strings_are_part_of_the_string_not_numeric() {
    let source = "let s = \"port 8080\";\n";
    let literals = extract_literals(source);
    assert_eq!(literals.len(), 1, "got {literals:?}");
    assert_eq!(literals[0].kind, LiteralKind::String);
}

#[test]
fn literals_inside_comments_are_ignored() {
    assert!(extract_literals("// 8080 and \"x\"\n").is_empty());
    assert!(extract_literals("# 8080 and \"x\"\n").is_empty());
}

#[test]
fn signed_and_suffixed_numbers_are_single_literals() {
    let literals = extract_literals("let t = -30i64;\n");
    assert_eq!(literals.len(), 1, "got {literals:?}");
    assert_eq!(literals[0].value, "-30i64");
}

#[test]
fn rust_lifetimes_are_not_string_literals() {
    let source = "fn f<'a>(x: &'a str) -> &'a str { x }\n";
    assert!(
        extract_literals(source).is_empty(),
        "lifetimes must not be read as literals"
    );
}

#[test]
fn rust_char_literal_is_reported_as_a_string() {
    let literals = extract_literals("let c = 'x';\n");
    assert_eq!(literals.len(), 1, "got {literals:?}");
    assert_eq!(literals[0].kind, LiteralKind::String);
    assert_eq!(literals[0].value, "x");
}
