//! Inline exception header validation.

use crate::storage::ProjectExceptions;
use std::path::Path;

/// Checks for inline exception headers: `// slugrails-exception: token=...; guard=...;`
#[must_use]
pub fn has_valid_exception(
    file: &Path,
    guard_id: &str,
    content: &str,
    exceptions: &ProjectExceptions,
) -> bool {
    // Scan up to first 50 lines to account for long license headers
    for line in content.lines().take(50) {
        if line.contains("slugrails-exception:")
            && line.contains("token=")
            && let Some(token_part) = line.split("token=").nth(1)
        {
            let token = token_part.split(';').next().unwrap_or("").trim();
            if exceptions.is_exception_valid(file, guard_id, token) {
                return true;
            }
        }
    }
    false
}
