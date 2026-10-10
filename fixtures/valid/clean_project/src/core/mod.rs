//! Core module for clean sample fixture.

use crate::foundation::sanitize;

/// Processes and validates an input token.
pub fn process_token(raw: &str) -> Option<String> {
    let cleaned = sanitize(raw);
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}
