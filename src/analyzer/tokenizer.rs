//! Unified pure-Rust code scanner and tokenizer.
//!
//! Strips comments, docstrings, and string literals in a single clean pass.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexState {
    Normal,
    LineComment,
    BlockComment,
    StringLiteral { escaped: bool },
    CharLiteral { escaped: bool },
}

use std::borrow::Cow;

/// A line in a source file, split into raw content and code-only content (comments/strings stripped).
#[derive(Debug, Clone)]
pub struct StrippedLine<'a> {
    pub line_number: usize,
    pub raw: Cow<'a, str>,
    pub code_only: Cow<'a, str>,
    pub has_code_token: bool,
}

/// Tokenizes source text and strips all non-executable tokens (comments, strings).
#[must_use]
pub fn tokenize_source(source: &str) -> Vec<StrippedLine<'_>> {
    let mut state = LexState::Normal;
    let mut lines = Vec::new();

    for (idx, raw_line) in source.lines().enumerate() {
        if state == LexState::Normal
            && !raw_line.contains('/')
            && !raw_line.contains('#')
            && !raw_line.contains('"')
            && !raw_line.contains('\'')
        {
            let has_code = !raw_line.trim().is_empty();
            lines.push(StrippedLine {
                line_number: idx + 1,
                raw: Cow::Borrowed(raw_line),
                code_only: Cow::Borrowed(raw_line),
                has_code_token: has_code,
            });
            continue;
        }

        let mut code_buf = String::with_capacity(raw_line.len());
        let mut chars = raw_line.chars().peekable();
        let mut has_code = false;

        while let Some(c) = chars.next() {
            let next = chars.peek().copied();

            match state {
                LexState::Normal => {
                    let is_rust_attr = c == '#' && matches!(next, Some('[' | '!'));
                    if (c == '/' && next == Some('/')) || (c == '#' && !is_rust_attr) {
                        state = LexState::LineComment;
                        break;
                    } else if is_rust_attr {
                        has_code = true;
                        code_buf.push(c);
                    } else if c == '/' && next == Some('*') {
                        state = LexState::BlockComment;
                        chars.next();
                    } else if c == '"' {
                        state = LexState::StringLiteral { escaped: false };
                        has_code = true;
                    } else if c == '\'' {
                        state = LexState::CharLiteral { escaped: false };
                        has_code = true;
                    } else {
                        if !c.is_whitespace() {
                            has_code = true;
                        }
                        code_buf.push(c);
                    }
                }
                LexState::LineComment => break,
                LexState::BlockComment => {
                    if c == '*' && next == Some('/') {
                        state = LexState::Normal;
                        chars.next();
                    }
                }
                LexState::StringLiteral { escaped } => {
                    if escaped {
                        state = LexState::StringLiteral { escaped: false };
                    } else if c == '\\' {
                        state = LexState::StringLiteral { escaped: true };
                    } else if c == '"' {
                        state = LexState::Normal;
                    }
                }
                LexState::CharLiteral { escaped } => {
                    if escaped {
                        state = LexState::CharLiteral { escaped: false };
                    } else if c == '\\' {
                        state = LexState::CharLiteral { escaped: true };
                    } else if c == '\'' {
                        state = LexState::Normal;
                    }
                }
            }
        }

        if state == LexState::LineComment {
            state = LexState::Normal;
        }

        let code_only = if code_buf == raw_line {
            Cow::Borrowed(raw_line)
        } else {
            Cow::Owned(code_buf)
        };

        lines.push(StrippedLine {
            line_number: idx + 1,
            raw: Cow::Borrowed(raw_line),
            code_only,
            has_code_token: has_code,
        });
    }

    lines
}

/// Counts non-comment, non-empty code lines in source text.
#[must_use]
pub fn count_code_lines(source: &str) -> usize {
    tokenize_source(source)
        .into_iter()
        .filter(|l| l.has_code_token)
        .count()
}

/// Checks if a source file contains `unwrap()` or `expect()` calls outside comments and strings.
#[must_use]
pub fn find_unwrap_expect_calls(source: &str) -> Vec<(usize, String)> {
    let mut results = Vec::new();
    for line in tokenize_source(source) {
        if line.code_only.contains(".unwrap()") {
            results.push((line.line_number, "bare .unwrap() call detected".to_string()));
        } else if line.code_only.contains(".expect(") {
            results.push((line.line_number, "bare .expect() call detected".to_string()));
        }
    }
    results
}

/// Checks for debug prints like dbg!, console.log outside comments/strings.
#[must_use]
pub fn find_debug_prints(source: &str) -> Vec<(usize, String)> {
    let mut results = Vec::new();
    let debug_patterns = ["dbg!", "console.log"];

    for line in tokenize_source(source) {
        for pat in debug_patterns {
            if line.code_only.contains(pat) && !line.raw.contains("tracing::") {
                results.push((line.line_number, format!("debug statement '{pat}' found")));
            }
        }
    }

    results
}

pub use super::imports::extract_imported_modules;
