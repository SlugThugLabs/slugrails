//! Stub, action-item and swallowed-error markers.

use super::lex::{CodeView, LazyRegex, raw_line, re, scan};
use crate::analyzer::tokenizer::tokenize_source;
use regex::Regex;
use std::sync::LazyLock;

/// What kind of marker was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerKind {
    /// `todo!(…)`
    Todo,
    /// `unimplemented!(…)`
    Unimplemented,
    /// `unreachable!(…)` — reported for every call; Python's guard only rejects
    /// ones carrying a placeholder message, so that policy belongs to the guard.
    Unreachable,
    /// `panic!("…")` whose message is a placeholder phrase.
    PanicNotImplemented,
    /// Python `raise NotImplementedError`.
    NotImplementedError,
    /// `TODO`/`FIXME`/`HACK`/`ACTION` in a comment.
    ActionItem,
    /// `let _ = …`
    DiscardedResult,
    /// `_ => {}` or `Err(_) => {}`
    EmptyMatchArm,
    /// `if let Err(_)`
    IgnoredError,
    /// Empty `catch {}` / `except: pass`.
    EmptyCatch,
}

/// A textual marker at a specific line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkerSite {
    /// 1-based line of the marker.
    pub line: usize,
    /// Which marker was found.
    pub kind: MarkerKind,
    /// The matched text (macro call, keyword, or arm).
    pub text: String,
    /// The trimmed raw line (for comment markers, the comment text).
    pub context: String,
}

static RE_ACTION: LazyRegex = LazyLock::new(|| Regex::new(r"\b(TODO|FIXME|HACK|ACTION)\b").ok());
static RE_TODO: LazyRegex = LazyLock::new(|| Regex::new(r"\btodo!\s*\(").ok());
static RE_UNIMPLEMENTED: LazyRegex = LazyLock::new(|| Regex::new(r"\bunimplemented!\s*\(").ok());
static RE_UNREACHABLE: LazyRegex = LazyLock::new(|| Regex::new(r"\bunreachable!\s*\(").ok());
static RE_PANIC_STR: LazyRegex = LazyLock::new(|| Regex::new(r#"panic!\s*\(\s*"([^"]*)""#).ok());
static RE_NOT_IMPLEMENTED: LazyRegex = LazyLock::new(|| Regex::new(r"\braise\s+NotImplementedError\b").ok());
static RE_PLACEHOLDER_WORDS: LazyRegex = LazyLock::new(|| Regex::new(r"(?i)(not implemented|todo|unimplemented|stub|placeholder|replace me)").ok());
static RE_LET_UNDERSCORE: LazyRegex = LazyLock::new(|| Regex::new(r"\blet\s+_\s*=").ok());
static RE_EMPTY_ARM: LazyRegex = LazyLock::new(|| Regex::new(r"_\s*=>\s*\{\s*\}").ok());
static RE_EMPTY_ERR_ARM: LazyRegex = LazyLock::new(|| Regex::new(r"Err\(\s*_\s*\)\s*=>\s*\{\s*\}").ok());
static RE_IF_LET_ERR: LazyRegex = LazyLock::new(|| Regex::new(r"\bif\s+let\s+Err\(\s*_\s*\)").ok());
static RE_BARE_EXCEPT: LazyRegex = LazyLock::new(|| Regex::new(r"except\s*:\s*pass").ok());
static RE_TYPED_EXCEPT: LazyRegex = LazyLock::new(|| Regex::new(r"except\s+[A-Za-z_][A-Za-z0-9_.]*\s*:\s*pass").ok());
static RE_EMPTY_CATCH: LazyRegex = LazyLock::new(|| Regex::new(r"catch\s*(?:\([^)]*\))?\s*\{\s*\}").ok());
static RE_EMPTY_PROMISE_CATCH: LazyRegex = LazyLock::new(|| Regex::new(r"\.catch\(\s*\(\s*\)\s*=>\s*\{\s*\}\s*\)").ok());

// ── find_stub_markers ────────────────────────────────────────────────────

/// Placeholder implementations: `todo!`, `unimplemented!`, `unreachable!`,
/// `panic!("not implemented")` and Python's `raise NotImplementedError`.
///
/// Matched against the **code-only** view, so a stub mentioned in a comment or
/// string is not reported. `unreachable!` is reported for every call — Python's
/// guard rejects only the ones carrying a placeholder message, and that
/// filtering belongs to the guard.
#[must_use]
pub fn find_stub_markers(source: &str) -> Vec<MarkerSite> {
    let mut sites = Vec::new();
    let raw_lines: Vec<&str> = source.lines().collect();
    let simple_stubs = [
        (&RE_TODO, MarkerKind::Todo, "todo!()"),
        (&RE_UNIMPLEMENTED, MarkerKind::Unimplemented, "unimplemented!()"),
        (&RE_UNREACHABLE, MarkerKind::Unreachable, "unreachable!()"),
        (&RE_NOT_IMPLEMENTED, MarkerKind::NotImplementedError, "raise NotImplementedError"),
    ];

    for stripped in tokenize_source(source) {
        let line = stripped.line_number;
        let context = raw_line(&raw_lines, line);
        for (slot, kind, text) in &simple_stubs {
            if re(slot).is_some_and(|pattern| pattern.is_match(&stripped.code_only)) {
                sites.push(MarkerSite {
                    line,
                    kind: *kind,
                    text: (*text).to_string(),
                    context: context.clone(),
                });
            }
        }

        if stripped.code_only.contains("panic!")
            && let Some(captures) =
                re(&RE_PANIC_STR).and_then(|pattern| pattern.captures(&stripped.raw))
            && let Some(message) = captures.get(1)
            && re(&RE_PLACEHOLDER_WORDS).is_some_and(|pattern| pattern.is_match(message.as_str()))
        {
            sites.push(MarkerSite {
                line,
                kind: MarkerKind::PanicNotImplemented,
                text: captures
                    .get(0)
                    .map_or_else(|| "panic!".to_string(), |whole| whole.as_str().to_string()),
                context: context.clone(),
            });
        }
    }
    sites
}

// ── find_action_markers ──────────────────────────────────────────────────

/// `TODO` / `FIXME` / `HACK` / `ACTION` inside comments, in both `//` and `#`
/// styles.
///
/// Matched against **comment text**, which is where these markers live; a bare
/// keyword in a string literal is not a comment and is not reported. Python's
/// guard additionally requires a `(#123):` issue link (`require_issue` default
/// `true`) — that requirement is policy and stays in the guard.
#[must_use]
pub fn find_action_markers(source: &str) -> Vec<MarkerSite> {
    let mut sites = Vec::new();
    let Some(pattern) = re(&RE_ACTION) else {
        return sites;
    };
    for (line, comment) in scan(source).comments {
        for found in pattern.find_iter(&comment) {
            sites.push(MarkerSite {
                line,
                kind: MarkerKind::ActionItem,
                text: found.as_str().to_string(),
                context: comment.clone(),
            });
        }
    }
    sites
}

// ── find_swallowed_errors ────────────────────────────────────────────────

/// Error-swallowing constructs: `let _ =`, `_ => {}`, `Err(_) => {}`,
/// `if let Err(_)`, empty `catch {}` and Python's `except: pass`.
///
/// Matched against the **code-only** joined text (patterns may span lines), so
/// these shapes quoted in a comment or string are not reported. Python's guard
/// skips test files entirely — a guard-level decision.
#[must_use]
pub fn find_swallowed_errors(source: &str) -> Vec<MarkerSite> {
    let patterns: [(&'static LazyRegex, MarkerKind); 8] = [
        (&RE_LET_UNDERSCORE, MarkerKind::DiscardedResult),
        (&RE_EMPTY_ERR_ARM, MarkerKind::EmptyMatchArm),
        (&RE_EMPTY_ARM, MarkerKind::EmptyMatchArm),
        (&RE_IF_LET_ERR, MarkerKind::IgnoredError),
        (&RE_BARE_EXCEPT, MarkerKind::EmptyCatch),
        (&RE_TYPED_EXCEPT, MarkerKind::EmptyCatch),
        (&RE_EMPTY_CATCH, MarkerKind::EmptyCatch),
        (&RE_EMPTY_PROMISE_CATCH, MarkerKind::EmptyCatch),
    ];
    let code = CodeView::build(source);
    let raw_lines: Vec<&str> = source.lines().collect();
    let mut sites = Vec::new();
    for (slot, kind) in patterns {
        let Some(pattern) = re(slot) else {
            continue;
        };
        for found in pattern.find_iter(&code.text) {
            let line = code.line_at(found.start());
            sites.push(MarkerSite {
                line,
                kind,
                text: found.as_str().trim().to_string(),
                context: raw_line(&raw_lines, line),
            });
        }
    }
    sites.sort_by(|left, right| {
        left.line
            .cmp(&right.line)
            .then_with(|| left.text.cmp(&right.text))
    });
    sites
}

