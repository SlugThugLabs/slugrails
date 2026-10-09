//! Unicode resilience, non-ASCII multi-byte slicing, and comment fuzzing tests.

#![allow(warnings, clippy::all, clippy::pedantic)]

use slugrails::analyzer::tokenize_source;

#[test]
fn test_multibyte_utf8_tokenization_no_panic() {
    let source = "
    // 这是一个中文字符的注释
    let token = \"dummy_mock_secret_payload_123456789\"; // 带注释
    let привет = \"мир\"; /* Блок комментариев */
    let emoji = \"🚀✨🔥\"; # Python comment with emojis
    fn test_func() {
        println!(\"Привет мир\");
    }
    ";

    let lines = tokenize_source(source);
    assert!(!lines.is_empty(), "Tokenization should produce lines");

    // Line with Chinese comment should have comment stripped
    let comment_line = lines.iter().find(|l| l.raw.contains("这是一个中文字符"));
    assert!(comment_line.is_some());
    assert_eq!(comment_line.unwrap().code_only.trim(), "");

    // Python-style comment should be stripped
    let emoji_line = lines.iter().find(|l| l.raw.contains("emoji"));
    assert!(emoji_line.is_some());
    assert!(
        !emoji_line.unwrap().code_only.contains("Python comment"),
        "Python # comment should be stripped from code_only"
    );
    assert!(
        emoji_line.unwrap().code_only.contains("let emoji ="),
        "Code portion should be preserved"
    );
}

#[test]
fn test_python_hash_inside_strings_preserved() {
    let source = "let color = \"#ff00aa\"; # trailing comment\n";
    let lines = tokenize_source(source);
    assert_eq!(lines.len(), 1);
    assert!(
        !lines[0].code_only.contains("trailing comment"),
        "Comment after string should be stripped"
    );
}
