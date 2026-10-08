//! Function spans and doc-comment detection, for Rust `fn` and Python `def`.

mod python;
mod rust;

pub use rust::has_doc_comment;

/// A function or method definition and its source extent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSpan {
    /// Function name.
    pub name: String,
    /// 1-based line of the signature.
    pub start_line: usize,
    /// 1-based line of the closing brace, or of the trailing `;` for a
    /// declaration without a body.
    pub end_line: usize,
    /// Parameters at brace-nesting depth zero, comma-separated (a trailing comma
    /// is ignored). Includes `self`/`&self` for methods, matching Python's
    /// `_count_params`.
    pub param_count: usize,
    /// Rust: the signature was preceded by `pub`. Python: the name does not
    /// start with `_`.
    pub is_pub: bool,
    /// Rust: a `///` or `#[doc]` comment precedes the item (attributes and blank
    /// lines between are skipped, up to five lines). Python: the first statement
    /// in the body is a string literal (a docstring).
    pub has_doc: bool,
}

/// Extracts function spans (Rust `fn`, Python `def`).
///
/// Spans are sorted by start line. Rust functions at minimum; a function whose
/// body cannot be located is still reported with the extent of its signature.
#[must_use]
pub fn function_spans(source: &str) -> Vec<FunctionSpan> {
    let mut spans = rust::rust_function_spans(source);
    spans.extend(python::python_function_spans(source));
    spans.sort_by(|left, right| {
        left.start_line
            .cmp(&right.start_line)
            .then_with(|| left.name.cmp(&right.name))
    });
    spans
}
