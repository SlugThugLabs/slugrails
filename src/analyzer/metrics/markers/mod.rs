//! Stub, action-item and swallowed-error markers.

mod stubs;
mod swallowed;
mod types;

pub use stubs::find_stub_markers;
pub use swallowed::find_swallowed_errors;
pub use types::{MarkerKind, MarkerSite};

use crate::analyzer::metrics::lex::{LazyRegex, re, scan};
use regex::Regex;
use std::sync::LazyLock;

static RE_ACTION: LazyRegex = LazyLock::new(|| Regex::new(r"\b(TODO|FIXME|HACK|ACTION)\b").ok());

/// `TODO` / `FIXME` / `HACK` / `ACTION` inside comments, in both `//` and `#`
/// styles.
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
