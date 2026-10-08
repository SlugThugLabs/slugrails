//! File context shared across guard checks.

use crate::library::catalog::GuardCatalogEntry;
use crate::storage::ProjectExceptions;
use std::path::Path;

/// Per-file state shared by the individual guard checks.
pub struct FileContext<'a> {
    pub file: &'a Path,
    pub rel_file: &'a Path,
    pub rel_str: &'a str,
    pub content: &'a str,
    /// The whole file is a test file (path convention) — every line is test code.
    pub is_test: bool,
    /// Line ranges of in-file `#[cfg(test)]` blocks.
    pub test_regions: &'a [(usize, usize)],
    pub rule_name: &'a str,
    pub guard: &'a GuardCatalogEntry,
    pub exceptions: &'a ProjectExceptions,
}

impl FileContext<'_> {
    /// Whether `line` holds test-only code, per the path convention or an
    /// in-file `#[cfg(test)]` region.
    #[must_use]
    pub fn is_test_line(&self, line: usize) -> bool {
        self.is_test
            || self
                .test_regions
                .iter()
                .any(|(start, end)| line >= *start && line <= *end)
    }
}
