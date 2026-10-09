//! The code-only source view and the raw lexical scan.
//!
//! [`CodeView`] is the tokenizer's `code_only` output with per-line offsets; [`scan`]
//! additionally recovers string/numeric literals and comment text from the raw
//! source.

mod numbers;
mod strings;

use super::literals::{LiteralKind, LiteralSite};
use crate::analyzer::tokenizer::tokenize_source;
use numbers::scan_number_at;
use regex::Regex;
use std::sync::LazyLock;
use strings::{looks_rust_like, scan_quoted};

/// A lazily compiled regex. `None` only if the literal pattern fails to compile,
/// which keeps a malformed pattern from reaching an `unwrap`/`expect` call.
pub(crate) type LazyRegex = LazyLock<Option<Regex>>;

/// Borrow the compiled regex out of a [`LazyRegex`].
pub(crate) fn re(slot: &LazyRegex) -> Option<&Regex> {
    slot.as_ref()
}

// ── Code/literal lexer ──────────────────────────────────────────────────────

/// The code-only view of a source file: one entry per line, plus a joined text
/// and per-line byte offsets for mapping a match back to its line.
pub(crate) struct CodeView {
    pub(crate) lines: Vec<String>,
    pub(crate) starts: Vec<usize>,
    pub(crate) text: String,
}

impl CodeView {
    #[must_use]
    pub(crate) fn build(source: &str) -> Self {
        let mut lines = Vec::new();
        let mut starts = Vec::new();
        let mut text = String::new();
        for stripped in tokenize_source(source) {
            starts.push(text.len());
            text.push_str(&stripped.code_only);
            text.push('\n');
            lines.push(stripped.code_only.into_owned());
        }
        Self {
            lines,
            starts,
            text,
        }
    }

    /// 1-based line containing `offset`.
    #[must_use]
    pub(crate) fn line_at(&self, offset: usize) -> usize {
        match self.starts.binary_search(&offset) {
            Ok(index) => index + 1,
            Err(index) => index.max(1),
        }
    }

    /// Byte offset where a 1-based line starts.
    #[must_use]
    pub(crate) fn line_start(&self, line: usize) -> usize {
        self.starts
            .get(line.saturating_sub(1))
            .copied()
            .unwrap_or(0)
    }
}

/// Literals and comment texts recovered by one pass over the raw source.
pub(crate) struct Scan {
    pub(crate) literals: Vec<LiteralSite>,
    /// `(line, trimmed comment text)` for every non-empty comment fragment.
    pub(crate) comments: Vec<(usize, String)>,
}

/// Lexes the raw source into literals and comment texts, tracking strings,
/// characters, raw strings and both comment styles so that `//` inside a URL
/// string or a digit inside a comment is not mistaken for code.
#[must_use]
pub(crate) fn scan(source: &str) -> Scan {
    let bytes = source.as_bytes();
    let raw_lines: Vec<&str> = source.lines().collect();
    let rust_like = looks_rust_like(source);
    let mut literals = Vec::new();
    let mut comments = Vec::new();
    let mut cursor = 0usize;
    let mut line = 1usize;

    while cursor < bytes.len() {
        let byte = bytes[cursor];

        if byte == b'\n' {
            line += 1;
            cursor += 1;
            continue;
        }
        if byte == b'/' && bytes.get(cursor + 1) == Some(&b'/') {
            let start = cursor + 2;
            let end = find_byte_from(bytes, start, b'\n').unwrap_or(bytes.len());
            push_comment(&mut comments, line, source.get(start..end).unwrap_or(""));
            cursor = end;
            continue;
        }
        if byte == b'/' && bytes.get(cursor + 1) == Some(&b'*') {
            if let Some(end) = find_pair_from(bytes, cursor + 2, *b"*/") {
                let interior = source.get(cursor + 2..end).unwrap_or("");
                for (offset, part) in interior.split('\n').enumerate() {
                    push_comment(&mut comments, line + offset, part);
                }
                line += interior.matches('\n').count();
                cursor = end + 2;
            } else {
                cursor = bytes.len();
            }
            continue;
        }
        // A `#` comment (Python/PyYAML) — but not a Rust attribute `#[…]`/`#![…]`.
        if byte == b'#' && !matches!(bytes.get(cursor + 1).copied(), Some(b'[' | b'!')) {
            let start = cursor + 1;
            let end = find_byte_from(bytes, start, b'\n').unwrap_or(bytes.len());
            push_comment(&mut comments, line, source.get(start..end).unwrap_or(""));
            cursor = end;
            continue;
        }
        if let Some((next, body_start, body_end)) = scan_quoted(bytes, cursor, rust_like) {
            literals.push(LiteralSite {
                line,
                kind: LiteralKind::String,
                value: source.get(body_start..body_end).unwrap_or("").to_string(),
                context: raw_line(&raw_lines, line),
            });
            line += source
                .get(cursor..next)
                .map_or(0, |consumed| consumed.matches('\n').count());
            cursor = next;
            continue;
        }
        if let Some(next) = scan_number_at(bytes, cursor) {
            literals.push(LiteralSite {
                line,
                kind: LiteralKind::Numeric,
                value: source.get(cursor..next).unwrap_or("").to_string(),
                context: raw_line(&raw_lines, line),
            });
            cursor = next;
            continue;
        }
        cursor += 1;
    }

    Scan { literals, comments }
}

/// Pushes a comment fragment unless it is only whitespace.
fn push_comment(comments: &mut Vec<(usize, String)>, line: usize, text: &str) {
    let trimmed = text.trim();
    if !trimmed.is_empty() {
        comments.push((line, trimmed.to_string()));
    }
}

/// The trimmed raw line at `line` (1-based), or an empty string past the end.
#[must_use]
pub(crate) fn raw_line(raw_lines: &[&str], line: usize) -> String {
    raw_lines
        .get(line.saturating_sub(1))
        .map_or_else(String::new, |text| text.trim().to_string())
}

/// First index at or after `from` holding `needle`.
pub(super) fn find_byte_from(bytes: &[u8], from: usize, needle: u8) -> Option<usize> {
    (from..bytes.len()).find(|index| bytes[*index] == needle)
}

/// First index at or after `from` where `needle` (a two-byte ASCII sequence) begins.
pub(super) fn find_pair_from(bytes: &[u8], from: usize, needle: [u8; 2]) -> Option<usize> {
    (from..bytes.len().saturating_sub(1))
        .find(|index| bytes[*index] == needle[0] && bytes[*index + 1] == needle[1])
}
