//! Detection of swallowed error patterns.

use super::types::{MarkerKind, MarkerSite};
use crate::analyzer::metrics::lex::{CodeView, LazyRegex, raw_line, re};
use regex::Regex;
use std::sync::LazyLock;

static RE_LET_UNDERSCORE: LazyRegex = LazyLock::new(|| Regex::new(r"\blet\s+_\s*=").ok());
static RE_EMPTY_ARM: LazyRegex = LazyLock::new(|| Regex::new(r"_\s*=>\s*\{\s*\}").ok());
static RE_EMPTY_ERR_ARM: LazyRegex =
    LazyLock::new(|| Regex::new(r"Err\(\s*_\s*\)\s*=>\s*\{\s*\}").ok());
static RE_IF_LET_ERR: LazyRegex = LazyLock::new(|| Regex::new(r"\bif\s+let\s+Err\(\s*_\s*\)").ok());
static RE_BARE_EXCEPT: LazyRegex = LazyLock::new(|| Regex::new(r"except\s*:\s*pass").ok());
static RE_TYPED_EXCEPT: LazyRegex =
    LazyLock::new(|| Regex::new(r"except\s+[A-Za-z_][A-Za-z0-9_.]*\s*:\s*pass").ok());
static RE_EMPTY_CATCH: LazyRegex =
    LazyLock::new(|| Regex::new(r"catch\s*(?:\([^)]*\))?\s*\{\s*\}").ok());
static RE_EMPTY_PROMISE_CATCH: LazyRegex =
    LazyLock::new(|| Regex::new(r"\.catch\(\s*\(\s*\)\s*=>\s*\{\s*\}\s*\)").ok());

/// Error-swallowing constructs: `let _ =`, `_ => {}`, `Err(_) => {}`,
/// `if let Err(_)`, empty `catch {}` and Python's `except: pass`.
#[must_use]
pub fn find_swallowed_errors(source: &str) -> Vec<MarkerSite> {
    let patterns: [(&'static LazyRegex, MarkerKind); 7] = [
        (&RE_LET_UNDERSCORE, MarkerKind::DiscardedResult),
        (&RE_EMPTY_ERR_ARM, MarkerKind::EmptyMatchArm),
        (&RE_EMPTY_ARM, MarkerKind::EmptyMatchArm),
        (&RE_IF_LET_ERR, MarkerKind::IgnoredError),
        (&RE_BARE_EXCEPT, MarkerKind::EmptyCatch),
        (&RE_TYPED_EXCEPT, MarkerKind::EmptyCatch),
        (&RE_EMPTY_CATCH, MarkerKind::EmptyCatch),
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
    if let Some(pattern) = re(&RE_EMPTY_PROMISE_CATCH) {
        for found in pattern.find_iter(&code.text) {
            let line = code.line_at(found.start());
            sites.push(MarkerSite {
                line,
                kind: MarkerKind::EmptyCatch,
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
