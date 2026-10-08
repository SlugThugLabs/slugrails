//! Stub detection for placeholder macros and errors.

use super::types::{MarkerKind, MarkerSite};
use crate::analyzer::metrics::lex::{LazyRegex, raw_line, re};
use crate::analyzer::tokenizer::tokenize_source;
use regex::Regex;
use std::sync::LazyLock;

static RE_TODO: LazyRegex = LazyLock::new(|| Regex::new(r"\btodo!\s*\(").ok());
static RE_UNIMPLEMENTED: LazyRegex = LazyLock::new(|| Regex::new(r"\bunimplemented!\s*\(").ok());
static RE_UNREACHABLE: LazyRegex = LazyLock::new(|| Regex::new(r"\bunreachable!\s*\(").ok());
static RE_PANIC_STR: LazyRegex = LazyLock::new(|| Regex::new(r#"panic!\s*\(\s*"([^"]*)""#).ok());
static RE_NOT_IMPLEMENTED: LazyRegex =
    LazyLock::new(|| Regex::new(r"\braise\s+NotImplementedError\b").ok());
static RE_PLACEHOLDER_WORDS: LazyRegex = LazyLock::new(|| {
    Regex::new(r"(?i)(not implemented|todo|unimplemented|stub|placeholder|replace me)").ok()
});

/// Placeholder implementations: `todo!`, `unimplemented!`, `unreachable!`,
/// `panic!("not implemented")` and Python's `raise NotImplementedError`.
#[must_use]
pub fn find_stub_markers(source: &str) -> Vec<MarkerSite> {
    let mut sites = Vec::new();
    let raw_lines: Vec<&str> = source.lines().collect();
    for stripped in tokenize_source(source) {
        let line = stripped.line_number;
        let context = raw_line(&raw_lines, line);
        if re(&RE_TODO).is_some_and(|pattern| pattern.is_match(&stripped.code_only)) {
            sites.push(MarkerSite {
                line,
                kind: MarkerKind::Todo,
                text: "todo!()".to_string(),
                context: context.clone(),
            });
        }
        if re(&RE_UNIMPLEMENTED).is_some_and(|pattern| pattern.is_match(&stripped.code_only)) {
            sites.push(MarkerSite {
                line,
                kind: MarkerKind::Unimplemented,
                text: "unimplemented!()".to_string(),
                context: context.clone(),
            });
        }
        if re(&RE_UNREACHABLE).is_some_and(|pattern| pattern.is_match(&stripped.code_only)) {
            sites.push(MarkerSite {
                line,
                kind: MarkerKind::Unreachable,
                text: "unreachable!()".to_string(),
                context: context.clone(),
            });
        }
        if re(&RE_NOT_IMPLEMENTED).is_some_and(|pattern| pattern.is_match(&stripped.code_only)) {
            sites.push(MarkerSite {
                line,
                kind: MarkerKind::NotImplementedError,
                text: "raise NotImplementedError".to_string(),
                context: context.clone(),
            });
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
