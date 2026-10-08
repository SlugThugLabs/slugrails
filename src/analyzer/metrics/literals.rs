//! Numeric and string literals.

use super::lex::scan;

/// Whether a literal is numeric or a quoted string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralKind {
    /// A numeric literal, including sign, radix prefix and suffix (`-30i64`).
    Numeric,
    /// A quoted string's inner text, with escapes left as written.
    String,
}

/// A numeric or string literal together with its location and line context.
///
/// Python's `magic_numbers` guard skips the values `0, 1, -1, 2, 10, 100, 1000`
/// and lines that are imports, constants or assertions; `hardcoded_values`
/// matches URLs, IPs, ports and timeouts and skips `const`/`static`/`import`
/// lines. Those are **guard policies** — this primitive reports every literal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteralSite {
    /// 1-based line where the literal starts.
    pub line: usize,
    /// Literal kind.
    pub kind: LiteralKind,
    /// Literal text: digits/sign/suffix for numerics, inner text for strings.
    pub value: String,
    /// The trimmed raw source line containing the literal.
    pub context: String,
}

// ── extract_literals ─────────────────────────────────────────────────────

/// Numeric and string literals in source order, each with its line and context.
///
/// Comments and string bodies are excluded from numeric scanning, so a port
/// number inside a comment is not a literal; conversely a number inside a string
/// is part of that string literal. Guard-level policy (magic-number skips,
/// hardcoded-value patterns) is deliberately not applied here.
#[must_use]
pub fn extract_literals(source: &str) -> Vec<LiteralSite> {
    scan(source).literals
}

