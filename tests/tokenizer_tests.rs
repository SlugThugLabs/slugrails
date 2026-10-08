use codeguards_mcp::analyzer::{
    count_code_lines, extract_imported_modules, find_unwrap_expect_calls, tokenize_source,
};

#[test]
fn commented_out_rust_imports_are_not_extracted() {
    let src = "// use crate::secret;\n// use super::danger;\nuse crate::real;\n";
    let imports = extract_imported_modules(src);
    assert_eq!(
        imports.len(),
        1,
        "only the live import must count: {imports:?}"
    );
    assert_eq!(imports[0].1, "crate::real");
}

#[test]
fn commented_out_python_imports_are_not_extracted() {
    let src = "# import evil\n# from evil import bad\nimport good\nfrom good import thing\n";
    let imports = extract_imported_modules(src);
    assert_eq!(
        imports.len(),
        2,
        "only live imports must count: {imports:?}"
    );
    assert_eq!(imports[0].1, "good");
    assert_eq!(imports[1].1, "good");
}

#[test]
fn block_comment_imports_are_not_extracted() {
    let src = "/* use crate::hidden;\n   import phantom */\nuse crate::live;\n";
    let imports = extract_imported_modules(src);
    assert_eq!(
        imports.len(),
        1,
        "block-comment imports must be stripped: {imports:?}"
    );
    assert_eq!(imports[0].1, "crate::live");
}

#[test]
fn block_comment_closes_at_first_terminator_like_rust_compilers() {
    let src = "/* outer /* inner */ still code */ let real = 1;\n";
    let lines = tokenize_source(src);
    assert!(
        lines[0].code_only.contains("let real = 1;"),
        "code after the first */ must be visible: {:?}",
        lines[0].code_only
    );
    assert!(
        !lines[0].code_only.contains("inner"),
        "comment interior must be stripped: {:?}",
        lines[0].code_only
    );
}

#[test]
fn block_comment_spanning_lines_strips_interior_and_restores_after() {
    let src = "let a = 1;\n/* multi\nline comment */\nlet b = 2;\n";
    let lines = tokenize_source(src);
    assert!(lines[0].code_only.contains("let a = 1;"));
    assert!(lines[1].code_only.is_empty());
    assert!(lines[2].code_only.is_empty());
    assert!(lines[3].code_only.contains("let b = 2;"));
}

#[test]
fn unwrap_inside_string_literal_is_not_detected() {
    let src = "let msg = \"call .unwrap() here\";\nlet s = \"a \\\".expect(x)\\\" b\";\n";
    assert!(find_unwrap_expect_calls(src).is_empty());
}

#[test]
fn unwrap_inside_comment_is_not_detected() {
    let src = "// let x = opt.unwrap();\n/* .expect(panics) */\n";
    assert!(find_unwrap_expect_calls(src).is_empty());
}

#[test]
fn unwrap_in_real_code_is_detected_with_line_numbers() {
    let src = "fn a() {}\nlet x = opt.unwrap();\nlet y = r.expect(\"nope\");\n";
    let hits = find_unwrap_expect_calls(src);
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].0, 2);
    assert_eq!(hits[1].0, 3);
}

#[test]
fn escaped_quote_in_string_does_not_terminate_early() {
    let src = "let s = \"he said \\\"hi\\\"\"; let x = ok.unwrap();\n";
    let lines = tokenize_source(src);
    assert!(
        lines[0].code_only.contains("let x = ok.unwrap();"),
        "escaped quote must keep string open until the true end: {:?}",
        lines[0].code_only
    );
}

#[test]
fn char_literal_with_escaped_quote() {
    let src = "let c = '\\''; let d = 'x';\n";
    let lines = tokenize_source(src);
    assert!(
        lines[0].code_only.contains("let d = "),
        "escaped char must parse: {:?}",
        lines[0].code_only
    );
}

#[test]
fn code_after_line_comment_is_stripped() {
    let src = "let a = 1; // trailing .unwrap() in comment\n";
    let lines = tokenize_source(src);
    assert!(!lines[0].code_only.contains("unwrap"));
    assert!(lines[0].code_only.contains("let a = 1;"));
}

#[test]
fn doc_comment_with_import_example_is_not_extracted() {
    let src = "/// Example: use crate::demo;\npub fn f() {}\n";
    let imports = extract_imported_modules(src);
    assert!(
        imports.is_empty(),
        "doc-comment example must not count: {imports:?}"
    );
}

#[test]
fn cfg_test_imports_are_extracted_like_all_imports() {
    let src = "#[cfg(test)]\nuse super::inner;\n";
    let imports = extract_imported_modules(src);
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].1, "super::inner");
}

#[test]
fn count_code_lines_ignores_comments_strings_and_blank() {
    let src = "// only comment\n\n/* block */\nlet real = \"a string\";\n";
    assert_eq!(count_code_lines(src), 1);
}

#[test]
fn string_only_line_counts_as_code() {
    assert_eq!(count_code_lines("\"just a string\";\n"), 1);
}
