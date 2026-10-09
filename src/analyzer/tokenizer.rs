//! Unified pure-Rust code scanner and tokenizer.
//!
//! Strips comments, docstrings, and string literals in a single clean pass.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LexState {
    Normal,
    LineComment,
    BlockComment,
    StringLiteral { escaped: bool },
    RawString { hashes: usize },
    CharLiteral { escaped: bool },
}

use super::patterns::{check_raw_closing, scan_char_or_lifetime, scan_raw_hashes};
use std::borrow::Cow;
use std::iter::Peekable;
use std::str::Chars;

/// A line in a source file, split into raw content and code-only content.
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
    source
        .lines()
        .enumerate()
        .map(|(idx, line)| tokenize_single_line(idx, line, &mut state))
        .collect()
}

fn tokenize_single_line<'a>(
    idx: usize,
    raw_line: &'a str,
    state: &mut LexState,
) -> StrippedLine<'a> {
    if *state == LexState::Normal
        && !raw_line.contains('/')
        && !raw_line.contains('#')
        && !raw_line.contains('"')
        && !raw_line.contains('\'')
    {
        return StrippedLine {
            line_number: idx + 1,
            raw: Cow::Borrowed(raw_line),
            code_only: Cow::Borrowed(raw_line),
            has_code_token: !raw_line.trim().is_empty(),
        };
    }

    let mut code_buf = String::with_capacity(raw_line.len());
    let mut chars = raw_line.chars().peekable();
    let mut has_code = false;
    let mut prev_char: Option<char> = None;

    while let Some(c) = chars.next() {
        let next = chars.peek().copied();
        match *state {
            LexState::Normal => {
                if handle_normal(
                    c,
                    next,
                    prev_char,
                    &mut chars,
                    &mut code_buf,
                    &mut has_code,
                    state,
                ) {
                    break;
                }
            }
            _ => handle_non_normal(c, next, &mut chars, state),
        }
        prev_char = Some(c);
    }

    if matches!(*state, LexState::LineComment | LexState::CharLiteral { .. }) {
        *state = LexState::Normal;
    }

    let code_only = if code_buf == raw_line {
        Cow::Borrowed(raw_line)
    } else {
        Cow::Owned(code_buf)
    };

    StrippedLine {
        line_number: idx + 1,
        raw: Cow::Borrowed(raw_line),
        code_only,
        has_code_token: has_code,
    }
}

fn handle_normal(
    c: char,
    next: Option<char>,
    prev: Option<char>,
    chars: &mut Peekable<Chars<'_>>,
    code: &mut String,
    has_code: &mut bool,
    state: &mut LexState,
) -> bool {
    let is_attr = c == '#' && matches!(next, Some('[' | '!'));
    if (c == '/' && next == Some('/')) || (c == '#' && !is_attr) {
        *state = LexState::LineComment;
        return true;
    }
    if is_attr {
        *has_code = true;
        code.push(c);
    } else if c == '/' && next == Some('*') {
        *state = LexState::BlockComment;
        chars.next();
    } else if c == 'r' && next == Some('"') {
        *state = LexState::RawString { hashes: 0 };
        chars.next();
        *has_code = true;
    } else if c == 'r' && next == Some('#') {
        scan_raw_hashes(c, chars, code, has_code, state);
    } else if c == '"' {
        *state = LexState::StringLiteral { escaped: false };
        *has_code = true;
    } else if c == '\'' {
        scan_char_or_lifetime(c, next, prev, code, has_code, state);
    } else {
        if !c.is_whitespace() {
            *has_code = true;
        }
        code.push(c);
    }
    false
}

fn handle_non_normal(
    c: char,
    next: Option<char>,
    chars: &mut Peekable<Chars<'_>>,
    state: &mut LexState,
) {
    match state {
        LexState::BlockComment => {
            if c == '*' && next == Some('/') {
                *state = LexState::Normal;
                chars.next();
            }
        }
        LexState::StringLiteral { escaped } => {
            if *escaped {
                *escaped = false;
            } else if c == '\\' {
                *escaped = true;
            } else if c == '"' {
                *state = LexState::Normal;
            }
        }
        LexState::RawString { hashes } => {
            if c == '"' && check_raw_closing(chars, *hashes) {
                *state = LexState::Normal;
            }
        }
        LexState::CharLiteral { escaped } => {
            if *escaped {
                *escaped = false;
            } else if c == '\\' {
                *escaped = true;
            } else if c == '\'' {
                *state = LexState::Normal;
            }
        }
        LexState::Normal | LexState::LineComment => {}
    }
}

pub use super::imports::extract_imported_modules;
pub use super::patterns::{count_code_lines, find_debug_prints, find_unwrap_expect_calls};
