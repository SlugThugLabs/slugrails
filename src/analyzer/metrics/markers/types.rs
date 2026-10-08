//! Marker site types and classifications.

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
