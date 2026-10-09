//! Source code pattern matching utilities for unwrap, debug prints, and line counting.

use super::tokenizer::{LexState, tokenize_source};
use std::iter::Peekable;
use std::str::Chars;

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

/// Checks for debug prints like `dbg!`, `console.log` outside comments/strings.
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

pub(crate) fn scan_raw_hashes(
    c: char,
    chars: &mut Peekable<Chars<'_>>,
    code: &mut String,
    has_code: &mut bool,
    state: &mut LexState,
) {
    let mut probe = chars.clone();
    probe.next();
    let mut h = 1;
    while let Some('#') = probe.next() {
        h += 1;
    }
    if let Some('"') = probe.next() {
        for _ in 0..=h {
            chars.next();
        }
        *state = LexState::RawString { hashes: h };
        *has_code = true;
    } else {
        *has_code = true;
        code.push(c);
    }
}

pub(crate) fn scan_char_or_lifetime(
    c: char,
    next: Option<char>,
    prev: Option<char>,
    code: &mut String,
    has_code: &mut bool,
    state: &mut LexState,
) {
    let is_lifetime = next.is_some_and(|n| n.is_alphabetic() || n == '_')
        && prev.is_none_or(|p| p.is_whitespace() || matches!(p, '<' | ',' | '&' | '(' | '[' | ':'));
    if is_lifetime {
        *has_code = true;
        code.push(c);
    } else {
        *state = LexState::CharLiteral { escaped: false };
        *has_code = true;
    }
}

pub(crate) fn check_raw_closing(chars: &mut Peekable<Chars<'_>>, hashes: usize) -> bool {
    let mut probe = chars.clone();
    for _ in 0..hashes {
        if probe.next() != Some('#') {
            return false;
        }
    }
    for _ in 0..hashes {
        chars.next();
    }
    true
}
