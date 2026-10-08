//! Rust function span extraction and doc-comment detection.

use super::FunctionSpan;
use crate::analyzer::metrics::lex::{CodeView, LazyRegex, re};
use regex::Regex;
use std::sync::LazyLock;

static RE_RUST_FN: LazyRegex =
    LazyLock::new(|| Regex::new(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)").ok());
static RE_PUB: LazyRegex = LazyLock::new(|| Regex::new(r"\bpub\b").ok());

pub fn rust_function_spans(source: &str) -> Vec<FunctionSpan> {
    let mut spans = Vec::new();
    let Some(re_fn) = re(&RE_RUST_FN) else {
        return spans;
    };
    let code = CodeView::build(source);
    for captures in re_fn.captures_iter(&code.text) {
        let Some(name_match) = captures.get(1) else {
            continue;
        };
        let Some(keyword_match) = captures.get(0) else {
            continue;
        };
        let start_line = code.line_at(keyword_match.start());
        let prefix = code
            .text
            .get(code.line_start(start_line)..keyword_match.start())
            .unwrap_or("");
        let is_pub = re(&RE_PUB).is_some_and(|pattern| pattern.is_match(prefix));
        let Some(open) = find_open_paren(&code.text, keyword_match.end()) else {
            continue;
        };
        let Some(close) = find_matching(&code.text, open, b'(', b')') else {
            continue;
        };
        let param_count = count_params(code.text.get(open + 1..close).unwrap_or(""));
        let end_line = match find_body_start(&code.text, close + 1) {
            Some(BodyStart::Brace(index)) => find_matching(&code.text, index, b'{', b'}')
                .map_or_else(|| code.line_at(index), |end| code.line_at(end)),
            Some(BodyStart::Semicolon(index)) => code.line_at(index),
            None => code.line_at(close),
        };
        spans.push(FunctionSpan {
            name: name_match.as_str().to_string(),
            start_line,
            end_line,
            param_count,
            is_pub,
            has_doc: has_doc_comment(source, start_line),
        });
    }
    spans
}

enum BodyStart {
    Brace(usize),
    Semicolon(usize),
}

fn find_body_start(text: &str, from: usize) -> Option<BodyStart> {
    let bytes = text.as_bytes();
    let mut depth = 0i64;
    let mut index = from;
    while index < bytes.len() {
        match bytes[index] {
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth -= 1,
            b'{' if depth == 0 => return Some(BodyStart::Brace(index)),
            b';' if depth == 0 => return Some(BodyStart::Semicolon(index)),
            _ => {}
        }
        index += 1;
    }
    None
}

fn find_open_paren(text: &str, from: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut angles = 0i64;
    let mut index = from;
    while index < bytes.len() {
        match bytes[index] {
            b'<' => angles += 1,
            b'>' if index > 0 && bytes[index - 1] != b'-' => angles -= 1,
            b'(' if angles <= 0 => return Some(index),
            b'\n' if angles <= 0 => {}
            _ => {}
        }
        index += 1;
    }
    None
}

pub(crate) fn find_matching(text: &str, open: usize, open_byte: u8, close_byte: u8) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0i64;
    let mut index = open;
    while index < bytes.len() {
        if bytes[index] == open_byte {
            depth += 1;
        } else if bytes[index] == close_byte {
            depth -= 1;
            if depth <= 0 {
                return Some(index);
            }
        }
        index += 1;
    }
    None
}

pub(crate) fn count_params(params: &str) -> usize {
    let trimmed = params.trim().trim_end_matches(',');
    if trimmed.is_empty() {
        return 0;
    }
    let mut depth = 0i64;
    let mut count = 1usize;
    for character in trimmed.chars() {
        match character {
            '<' | '(' | '[' | '{' => depth += 1,
            '>' | ')' | ']' | '}' => depth = (depth - 1).max(0),
            ',' if depth == 0 => count += 1,
            _ => {}
        }
    }
    count
}

#[must_use]
pub fn has_doc_comment(source: &str, start_line: usize) -> bool {
    let lines: Vec<&str> = source.lines().collect();
    if start_line == 0 || start_line > lines.len() {
        return false;
    }
    let mut index = start_line - 1;
    let mut checked = 0usize;
    while index > 0 && checked < 5 {
        let previous = lines[index - 1].trim();
        if previous.starts_with("///")
            || previous.starts_with("/**")
            || previous.starts_with("#[doc")
        {
            return true;
        }
        if previous.starts_with("#[") || previous.is_empty() {
            index -= 1;
            checked += 1;
            continue;
        }
        return false;
    }
    false
}
